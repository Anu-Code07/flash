//! Semantic analysis: type checking, effect analysis, and IR lowering.

mod provider;

use flash_ast::{
    Arg, Ast, AssignOp as AstAssignOp, AstNode, ComponentDef, Expr, ExprId as AstExprId,
    HandlerId as AstHandlerId, IncDecOp, InterpPart, Item, ScreenDef, Stmt, TypeRef,
};
use flash_ir::{
    AssignOp, ComponentIr, DepTable, HandlerBody, HandlerId, HandlerIr, IrExpr, ListenIr,
    NodeId, NodeIr, NodeKind, ParamDef, PropKey, ScreenIr, SlotId, StaticProp, UiIr, UpdateOp,
};
use flash_span::{Interner, Span, Symbol};
use flash_stl::{PrimaryProp, StlRegistry, TypeKind};
use flash_stl::widgets::widget_by_id;

use provider::{
    HandlerEnv, ProviderIndex, SlotKey, collect_expr_reads, compile_provider_actions,
    lower_provider_item, lower_screen_body_items, register_provider_slots, resolve_action_call,
};

pub struct Compiler {
    stl: StlRegistry,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub span: Span,
    pub suggestion: Option<String>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            stl: StlRegistry::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn compile(&mut self, ast: &Ast, interner: &mut Interner) -> Option<UiIr> {
        let provider_index = ProviderIndex::from_ast(ast);
        let mut screens = Vec::new();
        let mut components = Vec::new();
        let mut provider_irs = Vec::new();

        for item in &ast.items {
            if let Item::Provider(p) = item {
                provider_irs.push(lower_provider_item(p));
            }
        }

        for item in &ast.items {
            match item {
                Item::Screen(screen) => {
                    if let Some(ir) = self.lower_screen(ast, screen, &provider_index, interner) {
                        screens.push(ir);
                    }
                }
                Item::Component(comp) => {
                    if let Some(ir) = self.lower_component(ast, comp, interner) {
                        components.push(ir);
                    }
                }
                Item::Import(_) | Item::Provider(_) => {}
            }
        }

        if self.diagnostics.iter().any(|d| {
            d.code.starts_with("UI1") || d.code.starts_with("UI2") || d.code.starts_with("UI3")
        }) {
            return None;
        }

        Some(UiIr {
            screens,
            components,
            providers: provider_irs,
        })
    }

    fn lower_screen(
        &mut self,
        ast: &Ast,
        screen: &ScreenDef,
        provider_index: &ProviderIndex,
        interner: &mut Interner,
    ) -> Option<ScreenIr> {
        let mut slots = Vec::new();
        let mut slot_map = std::collections::HashMap::new();
        let mut compiled_actions = Vec::new();

        for inject in &screen.injects {
            if let Some(provider) = provider_index.providers.get(&inject.ty.name) {
                register_provider_slots(
                    self,
                    ast,
                    &mut slots,
                    &mut slot_map,
                    inject,
                    provider,
                    interner,
                );
            } else {
                self.diagnostics.push(Diagnostic {
                    code: "UI2001".into(),
                    message: format!(
                        "unknown provider type `{}`",
                        interner.get(inject.ty.name)
                    ),
                    span: inject.span,
                    suggestion: Some("define @provider before the screen".into()),
                });
            }
        }

        for state in &screen.state {
            let ty = self.resolve_type(&state.ty, interner);
            let init = self.lower_expr_readonly(ast, &slots, &slot_map, state.init, interner);
            let key = SlotKey {
                prefix: None,
                field: state.name,
            };
            let slot = SlotId(slots.len() as u32);
            slots.push(flash_ir::SlotDef {
                name: state.name,
                ty,
                init,
            });
            slot_map.insert(key, slot);
        }

        let mut ctx = LowerCtx::new(interner, slots, slot_map);

        for inject in &screen.injects {
            if let Some(provider) = provider_index.providers.get(&inject.ty.name) {
                compiled_actions.extend(compile_provider_actions(
                    self,
                    ast,
                    &ctx,
                    inject.name,
                    provider,
                    interner,
                ));
            }
        }

        let handler_env = HandlerEnv {
            actions: compiled_actions.clone(),
            injects: screen.injects.clone(),
        };

        let mut node_builder = NodeBuilder::new();
        lower_screen_body_items(
            self,
            ast,
            &mut ctx,
            &mut node_builder,
            &screen.body,
            &handler_env,
            interner,
        );

        let mut listens = Vec::new();
        for listen in &screen.listens {
            let reads = collect_expr_reads(ast, &ctx, listen.expr);
            listens.push(ListenIr {
                reads,
                handler: HandlerId(listens.len() as u32),
            });
        }

        let deps = DepTable::build(ctx.slots.len(), &node_builder.update_ops);

        Some(ScreenIr {
            name: screen.name,
            slots: ctx.slots,
            nodes: node_builder.nodes,
            static_props: node_builder.static_props,
            update_ops: node_builder.update_ops,
            deps,
            handlers: node_builder.handlers,
            actions: compiled_actions,
            listens,
            exprs: node_builder.exprs,
        })
    }

    fn lower_component(
        &mut self,
        ast: &Ast,
        comp: &ComponentDef,
        interner: &Interner,
    ) -> Option<ComponentIr> {
        let mut ctx = LowerCtx::new(interner, Vec::new(), std::collections::HashMap::new());
        for param in &comp.params {
            ctx.params
                .insert(param.name, self.resolve_type(&param.ty, interner));
        }
        let handler_env = HandlerEnv {
            actions: Vec::new(),
            injects: Vec::new(),
        };
        let mut node_builder = NodeBuilder::new();
        for &node_id in &comp.body {
            self.lower_node(
                ast,
                &mut ctx,
                &mut node_builder,
                node_id,
                None,
                &handler_env,
                interner,
            );
        }
        Some(ComponentIr {
            name: comp.name,
            params: comp
                .params
                .iter()
                .map(|p| ParamDef {
                    name: p.name,
                    ty: self.resolve_type(&p.ty, interner),
                })
                .collect(),
            nodes: node_builder.nodes,
            static_props: node_builder.static_props,
            update_ops: node_builder.update_ops,
            handlers: node_builder.handlers,
            exprs: Vec::new(),
        })
    }

    pub(crate) fn lower_node(
        &mut self,
        ast: &Ast,
        ctx: &mut LowerCtx<'_>,
        builder: &mut NodeBuilder,
        node_id: flash_ast::NodeId,
        parent: Option<NodeId>,
        handler_env: &HandlerEnv,
        interner: &Interner,
    ) {
        let node = &ast.nodes[node_id.0 as usize];
        match node {
            AstNode::Element {
                kind,
                args,
                children,
                handler,
                ..
            } => {
                let kind_name = ctx.interner.get(*kind);
                let ir_kind = NodeKind::from_name(kind_name).unwrap_or(NodeKind::TEXT);
                let ir_node_id = builder.add_node(ir_kind, parent);

                for arg in args {
                    self.lower_element_arg(ast, ctx, builder, ir_node_id, &ir_kind, arg);
                }

                if let Some(handler_id) = handler {
                    self.lower_handler(
                        ast,
                        ctx,
                        builder,
                        ir_node_id,
                        *handler_id,
                        handler_env,
                        interner,
                    );
                }

                for &child_id in children {
                    self.lower_node(
                        ast,
                        ctx,
                        builder,
                        child_id,
                        Some(ir_node_id),
                        handler_env,
                        interner,
                    );
                }
            }
            AstNode::If { then_, .. } => {
                for &child_id in then_ {
                    self.lower_node(ast, ctx, builder, child_id, parent, handler_env, interner);
                }
            }
            AstNode::List { body, .. } => {
                for &child_id in body {
                    self.lower_node(ast, ctx, builder, child_id, parent, handler_env, interner);
                }
            }
            AstNode::Error => {}
        }
    }

    fn lower_element_arg(
        &mut self,
        ast: &Ast,
        ctx: &LowerCtx<'_>,
        builder: &mut NodeBuilder,
        node_id: NodeId,
        kind: &NodeKind,
        arg: &Arg,
    ) {
        let prop_key = widget_by_id(kind.id())
            .map(|w| match w.primary_prop {
                PrimaryProp::Text => Some(PropKey::Text),
                PrimaryProp::Title => Some(PropKey::Title),
                PrimaryProp::Src => Some(PropKey::Src),
                PrimaryProp::Value => Some(PropKey::Value),
                PrimaryProp::None => None,
            })
            .flatten();
        let Some(prop_key) = prop_key else {
            return;
        };

        let expr = &ast.exprs[arg.value.0 as usize];
        match expr {
            Expr::Interp(parts) => {
                let (ir_expr, reads) = self.lower_interp(ast, ctx, parts);
                let expr_id = flash_ir::ExprId(builder.exprs.len() as u32);
                builder.exprs.push(ir_expr);
                builder.update_ops.push(UpdateOp {
                    node: node_id,
                    key: prop_key,
                    expr: expr_id,
                    reads,
                });
            }
            Expr::Str(sym) => {
                builder.static_props.push(StaticProp {
                    node: node_id,
                    key: prop_key,
                    value: IrExpr::Str(ctx.interner.get(*sym).to_string()),
                });
            }
            _ => {}
        }
    }

    fn lower_handler(
        &mut self,
        ast: &Ast,
        ctx: &LowerCtx<'_>,
        builder: &mut NodeBuilder,
        node_id: NodeId,
        handler_id: AstHandlerId,
        handler_env: &HandlerEnv,
        interner: &Interner,
    ) {
        let handler = &ast.handlers[handler_id.0 as usize];
        let h_id = HandlerId(builder.handlers.len() as u32);

        for stmt in &handler.stmts {
            match stmt {
                Stmt::Call { callee, .. } => {
                    if let Some(action_idx) = resolve_action_call(
                        ast,
                        *callee,
                        &handler_env.actions,
                        &handler_env.injects,
                        interner,
                    ) {
                        let writes = handler_env.actions[action_idx as usize].writes.clone();
                        builder.handlers.push(HandlerIr {
                            id: h_id,
                            node: node_id,
                            writes,
                            body: HandlerBody::InvokeAction(action_idx),
                        });
                        builder.nodes[node_id.0 as usize].handler = Some(h_id);
                    }
                }
                Stmt::IncDec {
                    target,
                    op: IncDecOp::Inc,
                    ..
                } => {
                    if let Some(slot) = ctx.resolve_path(&target.path) {
                        builder.handlers.push(HandlerIr {
                            id: h_id,
                            node: node_id,
                            writes: vec![slot],
                            body: HandlerBody::Increment(slot),
                        });
                        builder.nodes[node_id.0 as usize].handler = Some(h_id);
                    } else {
                        self.error_ui1003(&target.path[0], target.span, interner);
                    }
                }
                Stmt::IncDec {
                    target,
                    op: IncDecOp::Dec,
                    ..
                } => {
                    if let Some(slot) = ctx.resolve_path(&target.path) {
                        builder.handlers.push(HandlerIr {
                            id: h_id,
                            node: node_id,
                            writes: vec![slot],
                            body: HandlerBody::Decrement(slot),
                        });
                        builder.nodes[node_id.0 as usize].handler = Some(h_id);
                    }
                }
                Stmt::Assign {
                    target,
                    op: AstAssignOp::Set,
                    value,
                    ..
                } => {
                    if let Some(slot) = ctx.resolve_path(&target.path) {
                        let ir_val = self.lower_expr(ast, ctx, *value);
                        builder.handlers.push(HandlerIr {
                            id: h_id,
                            node: node_id,
                            writes: vec![slot],
                            body: HandlerBody::Assign {
                                slot,
                                op: AssignOp::Set,
                                value: ir_val,
                            },
                        });
                        builder.nodes[node_id.0 as usize].handler = Some(h_id);
                    }
                }
                _ => {}
            }
        }
    }

    fn lower_interp(
        &mut self,
        ast: &Ast,
        ctx: &LowerCtx<'_>,
        parts: &[InterpPart],
    ) -> (IrExpr, Vec<SlotId>) {
        let mut exprs = Vec::new();
        let mut reads = Vec::new();

        for part in parts {
            match part {
                InterpPart::Text(sym) => {
                    exprs.push(IrExpr::Str(ctx.interner.get(*sym).to_string()));
                }
                InterpPart::Expr(expr_id) => {
                    let reads_here = collect_expr_reads(ast, ctx, *expr_id);
                    reads.extend(&reads_here);
                    for slot in reads_here {
                        exprs.push(IrExpr::Slot(slot));
                    }
                }
            }
        }

        reads.sort_by_key(|s: &SlotId| s.0);
        reads.dedup();
        (IrExpr::Concat(exprs), reads)
    }

    pub(crate) fn lower_expr(&self, ast: &Ast, ctx: &LowerCtx<'_>, expr_id: AstExprId) -> IrExpr {
        let expr = &ast.exprs[expr_id.0 as usize];
        match expr {
            Expr::Int(v) => IrExpr::Int(*v),
            Expr::Float(v) => IrExpr::Float(*v),
            Expr::Bool(v) => IrExpr::Bool(*v),
            Expr::Str(sym) => IrExpr::Str(ctx.interner.get(*sym).to_string()),
            Expr::Ident(sym) => ctx
                .resolve_path(&[*sym])
                .map(IrExpr::Slot)
                .unwrap_or(IrExpr::Error),
            Expr::Field { base, field, .. } => {
                if let Expr::Ident(prefix) = &ast.exprs[base.0 as usize] {
                    ctx.resolve_path(&[*prefix, *field])
                        .map(IrExpr::Slot)
                        .unwrap_or(IrExpr::Error)
                } else {
                    IrExpr::Error
                }
            }
            _ => IrExpr::Error,
        }
    }

    pub(crate) fn resolve_type(&self, ty: &TypeRef, interner: &Interner) -> TypeKind {
        let name = interner.get(ty.name);
        let generics = ty
            .generics
            .iter()
            .map(|g| self.resolve_type(g, interner))
            .collect::<Vec<_>>();
        self.stl
            .resolve_type(name, &generics)
            .map(|t| {
                if ty.optional {
                    TypeKind::Option(Box::new(t))
                } else {
                    t
                }
            })
            .unwrap_or(TypeKind::Error)
    }

    pub(crate) fn error_ui1003(&mut self, sym: &Symbol, span: Span, interner: &Interner) {
        self.diagnostics.push(Diagnostic {
            code: "UI1003".into(),
            message: format!("unknown variable `{}`", interner.get(*sym)),
            span,
            suggestion: None,
        });
    }
}

struct LowerCtx<'a> {
    slots: Vec<flash_ir::SlotDef>,
    slot_map: std::collections::HashMap<SlotKey, SlotId>,
    params: std::collections::HashMap<Symbol, TypeKind>,
    interner: &'a Interner,
}

impl<'a> LowerCtx<'a> {
    fn new(
        interner: &'a Interner,
        slots: Vec<flash_ir::SlotDef>,
        slot_map: std::collections::HashMap<SlotKey, SlotId>,
    ) -> Self {
        Self {
            slots,
            slot_map,
            params: std::collections::HashMap::new(),
            interner,
        }
    }
}

struct NodeBuilder {
    nodes: Vec<NodeIr>,
    static_props: Vec<StaticProp>,
    update_ops: Vec<UpdateOp>,
    handlers: Vec<HandlerIr>,
    exprs: Vec<IrExpr>,
}

impl NodeBuilder {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            static_props: Vec::new(),
            update_ops: Vec::new(),
            handlers: Vec::new(),
            exprs: Vec::new(),
        }
    }

    fn add_node(&mut self, kind: NodeKind, parent: Option<NodeId>) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(NodeIr {
            kind,
            parent,
            children: Vec::new(),
            handler: None,
        });
        if let Some(p) = parent {
            self.nodes[p.0 as usize].children.push(id);
        }
        id
    }
}
