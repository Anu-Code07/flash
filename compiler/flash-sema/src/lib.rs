//! Semantic analysis: type checking, effect analysis, and IR lowering.

use flash_ast::{
    Arg, Ast, AssignOp as AstAssignOp, AstNode, ComponentDef, Expr, ExprId as AstExprId,
    HandlerId as AstHandlerId, IncDecOp, InterpPart, Item, LValue, ScreenDef, StateDef, Stmt,
    TypeRef,
};
use flash_ir::{
    AssignOp, ComponentIr, DepTable, HandlerBody, HandlerId, HandlerIr, IrExpr, NodeId, NodeIr,
    NodeKind, ParamDef, PropKey, ScreenIr, SlotDef, SlotId, StaticProp, UiIr, UpdateOp,
};
use flash_span::{Interner, Span, Symbol};
use flash_stl::{StlRegistry, TypeKind};

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

    pub fn compile(&mut self, ast: &Ast, interner: &Interner) -> Option<UiIr> {
        let mut screens = Vec::new();
        let mut components = Vec::new();

        for item in &ast.items {
            match item {
                Item::Screen(screen) => {
                    if let Some(ir) = self.lower_screen(ast, screen, interner) {
                        screens.push(ir);
                    }
                }
                Item::Component(comp) => {
                    if let Some(ir) = self.lower_component(ast, comp, interner) {
                        components.push(ir);
                    }
                }
                Item::Import(_) => {}
            }
        }

        if self.diagnostics.iter().any(|d| {
            d.code.starts_with("UI1") || d.code.starts_with("UI2") || d.code.starts_with("UI3")
        }) {
            return None;
        }

        Some(UiIr { screens, components })
    }

    fn lower_screen(&mut self, ast: &Ast, screen: &ScreenDef, interner: &Interner) -> Option<ScreenIr> {
        let mut ctx = LowerCtx::new(interner);

        for state in &screen.state {
            let ty = self.resolve_type(&state.ty, interner);
            let init = self.lower_expr(ast, &ctx, state.init);
            let slot = SlotId(ctx.slots.len() as u32);
            ctx.slots.push(SlotDef {
                name: state.name,
                ty,
                init,
            });
            ctx.slot_map.insert(state.name, slot);
        }

        let mut node_builder = NodeBuilder::new();
        for &node_id in &screen.body {
            self.lower_node(ast, &mut ctx, &mut node_builder, node_id, None, interner);
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
            exprs: Vec::new(),
        })
    }

    fn lower_component(&mut self, ast: &Ast, comp: &ComponentDef, interner: &Interner) -> Option<ComponentIr> {
        let mut ctx = LowerCtx::new(interner);
        for param in &comp.params {
            ctx.params.insert(param.name, self.resolve_type(&param.ty, interner));
        }
        let mut node_builder = NodeBuilder::new();
        for &node_id in &comp.body {
            self.lower_node(ast, &mut ctx, &mut node_builder, node_id, None, interner);
        }
        Some(ComponentIr {
            name: comp.name,
            params: comp.params.iter().map(|p| ParamDef {
                name: p.name,
                ty: self.resolve_type(&p.ty, interner),
            }).collect(),
            nodes: node_builder.nodes,
            static_props: node_builder.static_props,
            update_ops: node_builder.update_ops,
            handlers: node_builder.handlers,
            exprs: Vec::new(),
        })
    }

    fn lower_node(
        &mut self,
        ast: &Ast,
        ctx: &mut LowerCtx<'_>,
        builder: &mut NodeBuilder,
        node_id: flash_ast::NodeId,
        parent: Option<NodeId>,
        interner: &Interner,
    ) {
        let node = &ast.nodes[node_id.0 as usize];
        match node {
            AstNode::Element { kind, args, children, handler, .. } => {
                let kind_name = ctx.interner.get(*kind);
                let ir_kind = NodeKind::from_name(kind_name).unwrap_or(NodeKind::Text);
                let ir_node_id = builder.add_node(ir_kind, parent);

                for arg in args {
                    self.lower_element_arg(ast, ctx, builder, ir_node_id, &ir_kind, arg);
                }

                if let Some(handler_id) = handler {
                    self.lower_handler(ast, ctx, builder, ir_node_id, *handler_id, interner);
                }

                for &child_id in children {
                    self.lower_node(ast, ctx, builder, child_id, Some(ir_node_id), interner);
                }
            }
            AstNode::If { then_, .. } => {
                for &child_id in then_ {
                    self.lower_node(ast, ctx, builder, child_id, parent, interner);
                }
            }
            AstNode::List { body, .. } => {
                for &child_id in body {
                    self.lower_node(ast, ctx, builder, child_id, parent, interner);
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
        let expr = &ast.exprs[arg.value.0 as usize];
        match (kind, expr) {
            (NodeKind::Text, Expr::Interp(parts)) => {
                let (_ir_expr, reads) = self.lower_interp(ast, ctx, parts);
                builder.update_ops.push(UpdateOp {
                    node: node_id,
                    key: PropKey::Text,
                    expr: flash_ir::ExprId(builder.update_ops.len() as u32),
                    reads,
                });
            }
            (NodeKind::Text, Expr::Str(sym)) => {
                builder.static_props.push(StaticProp {
                    node: node_id,
                    key: PropKey::Text,
                    value: IrExpr::Str(ctx.interner.get(*sym).to_string()),
                });
            }
            (NodeKind::Button, Expr::Str(sym)) => {
                builder.static_props.push(StaticProp {
                    node: node_id,
                    key: PropKey::Title,
                    value: IrExpr::Str(ctx.interner.get(*sym).to_string()),
                });
            }
            (NodeKind::Button, Expr::Interp(parts)) if parts.len() == 1 => {
                if let InterpPart::Text(sym) = &parts[0] {
                    builder.static_props.push(StaticProp {
                        node: node_id,
                        key: PropKey::Title,
                        value: IrExpr::Str(ctx.interner.get(*sym).to_string()),
                    });
                }
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
        interner: &Interner,
    ) {
        let handler = &ast.handlers[handler_id.0 as usize];
        let h_id = HandlerId(builder.handlers.len() as u32);

        for stmt in &handler.stmts {
            match stmt {
                Stmt::IncDec { target, op: IncDecOp::Inc, .. } => {
                    if let Some(slot) = ctx.resolve_lvalue(&target.path) {
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
                Stmt::Assign { target, op: AstAssignOp::Set, value, .. } => {
                    if let Some(slot) = ctx.resolve_lvalue(&target.path) {
                        let ir_val = self.lower_expr(ast, ctx, *value);
                        builder.handlers.push(HandlerIr {
                            id: h_id,
                            node: node_id,
                            writes: vec![slot],
                            body: HandlerBody::Assign { slot, op: AssignOp::Set, value: ir_val },
                        });
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
                    let reads_here = self.collect_reads(ast, ctx, *expr_id);
                    reads.extend(&reads_here);
                    for slot in reads_here {
                        exprs.push(IrExpr::Slot(slot));
                    }
                }
            }
        }

        (IrExpr::Concat(exprs), reads)
    }

    fn collect_reads(&mut self, ast: &Ast, ctx: &LowerCtx<'_>, expr_id: AstExprId) -> Vec<SlotId> {
        let expr = &ast.exprs[expr_id.0 as usize];
        match expr {
            Expr::Ident(sym) => {
                if let Some(s) = ctx.slot_map.get(sym) {
                    vec![*s]
                } else {
                    self.error_ui1003(sym, Span::default(), ctx.interner);
                    Vec::new()
                }
            }
            _ => Vec::new(),
        }
    }

    fn lower_expr(&self, ast: &Ast, ctx: &LowerCtx<'_>, expr_id: AstExprId) -> IrExpr {
        let expr = &ast.exprs[expr_id.0 as usize];
        match expr {
            Expr::Int(v) => IrExpr::Int(*v),
            Expr::Float(v) => IrExpr::Float(*v),
            Expr::Bool(v) => IrExpr::Bool(*v),
            Expr::Str(sym) => IrExpr::Str(ctx.interner.get(*sym).to_string()),
            Expr::Ident(sym) => ctx.slot_map.get(sym)
                .map(|s| IrExpr::Slot(*s))
                .unwrap_or(IrExpr::Error),
            _ => IrExpr::Error,
        }
    }

    fn resolve_type(&self, ty: &TypeRef, interner: &Interner) -> TypeKind {
        let name = interner.get(ty.name);
        let generics = ty.generics.iter().map(|g| self.resolve_type(g, interner)).collect::<Vec<_>>();
        self.stl.resolve_type(name, &generics)
            .map(|t| if ty.optional { TypeKind::Option(Box::new(t)) } else { t })
            .unwrap_or(TypeKind::Error)
    }

    fn error_ui1003(&mut self, sym: &Symbol, span: Span, interner: &Interner) {
        self.diagnostics.push(Diagnostic {
            code: "UI1003".into(),
            message: format!("unknown variable `{}`", interner.get(*sym)),
            span,
            suggestion: None,
        });
    }
}

struct LowerCtx<'a> {
    slots: Vec<SlotDef>,
    slot_map: std::collections::HashMap<Symbol, SlotId>,
    params: std::collections::HashMap<Symbol, TypeKind>,
    interner: &'a Interner,
}

impl<'a> LowerCtx<'a> {
    fn new(interner: &'a Interner) -> Self {
        Self {
            slots: Vec::new(),
            slot_map: std::collections::HashMap::new(),
            params: std::collections::HashMap::new(),
            interner,
        }
    }

    fn resolve_lvalue(&self, path: &[Symbol]) -> Option<SlotId> {
        path.first().and_then(|s| self.slot_map.get(s).copied())
    }
}

struct NodeBuilder {
    nodes: Vec<NodeIr>,
    static_props: Vec<StaticProp>,
    update_ops: Vec<UpdateOp>,
    handlers: Vec<HandlerIr>,
}

impl NodeBuilder {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            static_props: Vec::new(),
            update_ops: Vec::new(),
            handlers: Vec::new(),
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
