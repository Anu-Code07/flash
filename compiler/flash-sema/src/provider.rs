//! @provider lowering — Riverpod-style state with compile-time dependency tracking.

use std::collections::HashMap;

use flash_ast::{
    ActionDef, AssignOp as AstAssignOp, Ast, Expr, ExprId as AstExprId, HandlerId as AstHandlerId,
    IncDecOp, InjectDef, Item, MatchPattern, ProviderDef, ProviderScope, ScreenBodyItem, Stmt,
};
use flash_ir::{
    AssignOp, CompiledAction, HandlerBody, IrExpr, ProviderIr, SlotDef, SlotId,
};
use flash_span::{Interner, Symbol};
use flash_stl::TypeKind;

use crate::{Compiler, LowerCtx, NodeBuilder};

pub struct ProviderIndex {
    pub providers: HashMap<Symbol, ProviderDef>,
}

impl ProviderIndex {
    pub fn from_ast(ast: &Ast) -> Self {
        let mut providers = HashMap::new();
        for item in &ast.items {
            if let Item::Provider(p) = item {
                providers.insert(p.name, p.clone());
            }
        }
        Self { providers }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SlotKey {
    pub prefix: Option<Symbol>,
    pub field: Symbol,
}

pub fn slot_display_name(interner: &mut Interner, key: &SlotKey) -> Symbol {
    if let Some(prefix) = key.prefix {
        let s = format!("{}.{}", interner.get(prefix), interner.get(key.field));
        interner.intern(&s)
    } else {
        key.field
    }
}

pub fn infer_action_writes(
    ast: &Ast,
    handler_id: AstHandlerId,
    provider_prefix: Option<Symbol>,
) -> Vec<SlotKey> {
    let handler = &ast.handlers[handler_id.0 as usize];
    let mut writes = Vec::new();
    for stmt in &handler.stmts {
        collect_stmt_writes(stmt, provider_prefix, &mut writes);
    }
    writes.sort_by(|a, b| {
        a.prefix
            .map(|s| s.0)
            .cmp(&b.prefix.map(|s| s.0))
            .then_with(|| a.field.0.cmp(&b.field.0))
    });
    writes.dedup();
    writes
}

fn collect_stmt_writes(stmt: &Stmt, prefix: Option<Symbol>, out: &mut Vec<SlotKey>) {
    match stmt {
        Stmt::Assign { target, .. } | Stmt::IncDec { target, .. } => {
            if let Some(key) = lvalue_to_slot_key(&target.path, prefix) {
                out.push(key);
            }
        }
        Stmt::If { then_, else_, .. } => {
            for s in then_ {
                collect_stmt_writes(s, prefix, out);
            }
            if let Some(els) = else_ {
                for s in els {
                    collect_stmt_writes(s, prefix, out);
                }
            }
        }
        Stmt::Call { .. } | Stmt::Await { .. } => {}
    }
}

fn lvalue_to_slot_key(path: &[Symbol], inject_prefix: Option<Symbol>) -> Option<SlotKey> {
    if path.is_empty() {
        return None;
    }
    if path.len() == 1 {
        return Some(SlotKey {
            prefix: inject_prefix,
            field: path[0],
        });
    }
    if path.len() == 2 {
        return Some(SlotKey {
            prefix: Some(path[0]),
            field: path[1],
        });
    }
    None
}

pub fn lower_action_body(
    compiler: &mut Compiler,
    ast: &Ast,
    ctx: &LowerCtx<'_>,
    handler_id: AstHandlerId,
    inject_prefix: Option<Symbol>,
    interner: &Interner,
) -> Vec<HandlerBody> {
    let handler = &ast.handlers[handler_id.0 as usize];
    let mut ops = Vec::new();
    for stmt in &handler.stmts {
        if let Some(op) = lower_action_stmt(compiler, ast, ctx, stmt, inject_prefix, interner) {
            ops.push(op);
        }
    }
    ops
}

fn resolve_action_path(
    ctx: &LowerCtx<'_>,
    path: &[Symbol],
    inject_prefix: Option<Symbol>,
) -> Option<SlotId> {
    if path.len() == 1 {
        return ctx
            .slot_map
            .get(&SlotKey {
                prefix: inject_prefix,
                field: path[0],
            })
            .copied();
    }
    ctx.resolve_path(path)
}

fn lower_action_stmt(
    compiler: &mut Compiler,
    ast: &Ast,
    ctx: &LowerCtx<'_>,
    stmt: &Stmt,
    inject_prefix: Option<Symbol>,
    interner: &Interner,
) -> Option<HandlerBody> {
    match stmt {
        Stmt::IncDec {
            target,
            op: IncDecOp::Inc,
            ..
        } => resolve_action_path(ctx, &target.path, inject_prefix)
            .map(HandlerBody::Increment)
            .or_else(|| {
                compiler.error_ui1003(&target.path[0], target.span, interner);
                None
            }),
        Stmt::IncDec {
            target,
            op: IncDecOp::Dec,
            ..
        } => resolve_action_path(ctx, &target.path, inject_prefix)
            .map(HandlerBody::Decrement)
            .or_else(|| {
                compiler.error_ui1003(&target.path[0], target.span, interner);
                None
            }),
        Stmt::Assign {
            target,
            op: AstAssignOp::Set,
            value,
            ..
        } => {
            let slot = resolve_action_path(ctx, &target.path, inject_prefix)?;
            let ir_val = compiler.lower_expr(ast, ctx, *value);
            Some(HandlerBody::Assign {
                slot,
                op: AssignOp::Set,
                value: ir_val,
            })
        }
        Stmt::If { then_, else_, .. } => {
            let mut ops = Vec::new();
            for s in then_ {
                if let Some(op) = lower_action_stmt(compiler, ast, ctx, s, inject_prefix, interner) {
                    ops.push(op);
                }
            }
            if let Some(els) = else_ {
                for s in els {
                    if let Some(op) = lower_action_stmt(compiler, ast, ctx, s, inject_prefix, interner) {
                        ops.push(op);
                    }
                }
            }
            if ops.is_empty() {
                None
            } else if ops.len() == 1 {
                Some(ops[0].clone())
            } else {
                Some(HandlerBody::Sequence(ops))
            }
        }
        _ => None,
    }
}

pub fn register_provider_slots(
    compiler: &mut Compiler,
    ast: &Ast,
    slots: &mut Vec<SlotDef>,
    slot_map: &mut HashMap<SlotKey, SlotId>,
    inject: &InjectDef,
    provider: &ProviderDef,
    interner: &mut Interner,
) {
    let prefix = Some(inject.name);
    for state in &provider.states {
        let ty = compiler.resolve_type(&state.ty, interner);
        let init = compiler.lower_expr_readonly(ast, slots, slot_map, state.init, interner);
        let key = SlotKey {
            prefix,
            field: state.name,
        };
        let slot = alloc_slot(slots, slot_display_name(interner, &key), ty, init);
        slot_map.insert(key, slot);
    }
}

fn alloc_slot(slots: &mut Vec<SlotDef>, name: Symbol, ty: TypeKind, init: IrExpr) -> SlotId {
    let slot = SlotId(slots.len() as u32);
    slots.push(SlotDef { name, ty, init });
    slot
}

impl Compiler {
    pub(crate) fn lower_expr_readonly(
        &self,
        ast: &Ast,
        slots: &[SlotDef],
        slot_map: &HashMap<SlotKey, SlotId>,
        expr_id: AstExprId,
        interner: &Interner,
    ) -> IrExpr {
        let ctx = ReadonlyCtx { slots, slot_map, interner };
        self.lower_expr_with_ctx(ast, &ctx, expr_id)
    }

    fn lower_expr_with_ctx(
        &self,
        ast: &Ast,
        ctx: &ReadonlyCtx<'_>,
        expr_id: AstExprId,
    ) -> IrExpr {
        let expr = &ast.exprs[expr_id.0 as usize];
        match expr {
            Expr::Int(v) => IrExpr::Int(*v),
            Expr::Float(v) => IrExpr::Float(*v),
            Expr::Bool(v) => IrExpr::Bool(*v),
            Expr::Str(sym) => IrExpr::Str(ctx.interner.get(*sym).to_string()),
            Expr::Ident(sym) => resolve_path_readonly(ctx, &[*sym])
                .map(IrExpr::Slot)
                .unwrap_or(IrExpr::Error),
            _ => IrExpr::Error,
        }
    }
}

struct ReadonlyCtx<'a> {
    slots: &'a [SlotDef],
    slot_map: &'a HashMap<SlotKey, SlotId>,
    interner: &'a Interner,
}

fn resolve_path_readonly(ctx: &ReadonlyCtx<'_>, path: &[Symbol]) -> Option<SlotId> {
    if path.is_empty() {
        return None;
    }
    if path.len() == 1 {
        return ctx
            .slot_map
            .get(&SlotKey {
                prefix: None,
                field: path[0],
            })
            .copied();
    }
    if path.len() == 2 {
        return ctx
            .slot_map
            .get(&SlotKey {
                prefix: Some(path[0]),
                field: path[1],
            })
            .copied();
    }
    None
}

pub fn compile_provider_actions(
    compiler: &mut Compiler,
    ast: &Ast,
    ctx: &LowerCtx<'_>,
    inject_name: Symbol,
    provider: &ProviderDef,
    interner: &Interner,
) -> Vec<CompiledAction> {
    let prefix = Some(inject_name);
    provider
        .actions
        .iter()
        .map(|action| {
            compile_one_action(compiler, ast, ctx, action, prefix, provider.name, interner)
        })
        .collect()
}

fn compile_one_action(
    compiler: &mut Compiler,
    ast: &Ast,
    ctx: &LowerCtx<'_>,
    action: &ActionDef,
    prefix: Option<Symbol>,
    provider_name: Symbol,
    interner: &Interner,
) -> CompiledAction {
    let writes_keys = infer_action_writes(ast, action.handler, prefix);
    let writes = writes_keys
        .iter()
        .filter_map(|k| ctx.slot_map.get(k).copied())
        .collect();
    let body = lower_action_body(compiler, ast, ctx, action.handler, prefix, interner);
    CompiledAction {
        name: action.name,
        provider: Some(provider_name),
        writes,
        body,
        is_async: action.is_async,
    }
}

pub fn lower_provider_item(provider: &ProviderDef) -> ProviderIr {
    ProviderIr {
        name: provider.name,
        scope: match provider.scope {
            ProviderScope::Screen => flash_ir::ProviderScope::Screen,
            ProviderScope::App => flash_ir::ProviderScope::App,
            ProviderScope::Family => flash_ir::ProviderScope::Family,
        },
        state_fields: provider.states.iter().map(|s| s.name).collect(),
        actions: provider.actions.iter().map(|a| a.name).collect(),
    }
}

pub fn collect_expr_reads(
    ast: &Ast,
    ctx: &LowerCtx<'_>,
    expr_id: AstExprId,
) -> Vec<SlotId> {
    let expr = &ast.exprs[expr_id.0 as usize];
    match expr {
        Expr::Ident(sym) => ctx
            .slot_map
            .get(&SlotKey {
                prefix: None,
                field: *sym,
            })
            .map(|s| vec![*s])
            .unwrap_or_default(),
        Expr::Field { base, field, .. } => {
            if let Expr::Ident(prefix_sym) = &ast.exprs[base.0 as usize] {
                let key = SlotKey {
                    prefix: Some(*prefix_sym),
                    field: *field,
                };
                return ctx.slot_map.get(&key).map(|s| vec![*s]).unwrap_or_default();
            }
            Vec::new()
        }
        Expr::Interp(parts) => {
            let mut reads = Vec::new();
            for part in parts {
                if let flash_ast::InterpPart::Expr(e) = part {
                    reads.extend(collect_expr_reads(ast, ctx, *e));
                }
            }
            reads.sort_by_key(|s| s.0);
            reads.dedup();
            reads
        }
        _ => Vec::new(),
    }
}

pub fn resolve_action_call(
    ast: &Ast,
    expr_id: AstExprId,
    actions: &[CompiledAction],
    injects: &[InjectDef],
    interner: &Interner,
) -> Option<u32> {
    let expr = &ast.exprs[expr_id.0 as usize];
    if let Expr::Call { callee, .. } = expr {
        if let Expr::Field { base, field, .. } = &ast.exprs[callee.0 as usize] {
            if let Expr::Ident(inject_sym) = &ast.exprs[base.0 as usize] {
                let action_name = interner.get(*field);
                let inject_name = interner.get(*inject_sym);
                let inject_ty = injects
                    .iter()
                    .find(|i| interner.get(i.name) == inject_name)
                    .map(|i| i.ty.name);
                return actions
                    .iter()
                    .position(|a| interner.get(a.name) == action_name && a.provider == inject_ty)
                    .map(|i| i as u32);
            }
        }
    }
    None
}

pub struct HandlerEnv {
    pub actions: Vec<CompiledAction>,
    pub injects: Vec<InjectDef>,
}

pub fn lower_screen_body_items(
    compiler: &mut Compiler,
    ast: &Ast,
    ctx: &mut LowerCtx<'_>,
    builder: &mut NodeBuilder,
    items: &[ScreenBodyItem],
    handler_env: &HandlerEnv,
    interner: &Interner,
) {
    for item in items {
        match item {
            ScreenBodyItem::Node(node_id) => {
                compiler.lower_node(ast, ctx, builder, *node_id, None, handler_env, interner);
            }
            ScreenBodyItem::Match(match_def) => {
                let arm = match_def
                    .arms
                    .iter()
                    .find(|a| matches!(a.pattern, MatchPattern::Ident(_)))
                    .or_else(|| match_def.arms.first());
                if let Some(arm) = arm {
                    for &node_id in &arm.body {
                        compiler.lower_node(ast, ctx, builder, node_id, None, handler_env, interner);
                    }
                }
            }
        }
    }
}

impl LowerCtx<'_> {
    pub fn alloc_slot(&mut self, name: Symbol, ty: TypeKind, init: IrExpr) -> SlotId {
        let slot = SlotId(self.slots.len() as u32);
        self.slots.push(SlotDef { name, ty, init });
        slot
    }

    pub fn resolve_path(&self, path: &[Symbol]) -> Option<SlotId> {
        if path.is_empty() {
            return None;
        }
        if path.len() == 1 {
            return self
                .slot_map
                .get(&SlotKey {
                    prefix: None,
                    field: path[0],
                })
                .copied();
        }
        if path.len() == 2 {
            return self
                .slot_map
                .get(&SlotKey {
                    prefix: Some(path[0]),
                    field: path[1],
                })
                .copied();
        }
        None
    }
}
