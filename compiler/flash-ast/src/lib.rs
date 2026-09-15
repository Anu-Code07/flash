//! Abstract Syntax Tree for the Flash `.ui` language.

use flash_span::{Span, Symbol};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExprId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HandlerId(pub u32);

#[derive(Clone, Debug)]
pub struct Ast {
    pub items: Vec<Item>,
    pub nodes: Vec<AstNode>,
    pub exprs: Vec<Expr>,
    pub handlers: Vec<Handler>,
    pub spans: Vec<Span>,
}

impl Ast {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            nodes: Vec::new(),
            exprs: Vec::new(),
            handlers: Vec::new(),
            spans: Vec::new(),
        }
    }

    pub fn add_expr(&mut self, expr: Expr, span: Span) -> ExprId {
        let id = ExprId(self.exprs.len() as u32);
        self.exprs.push(expr);
        self.spans.push(span);
        id
    }

    pub fn add_node(&mut self, node: AstNode, span: Span) -> NodeId {
        let id = NodeId(self.nodes.len() as u32);
        self.nodes.push(node);
        self.spans.push(span);
        id
    }

    pub fn add_handler(&mut self, handler: Handler, span: Span) -> HandlerId {
        let id = HandlerId(self.handlers.len() as u32);
        self.handlers.push(handler);
        self.spans.push(span);
        id
    }
}

#[derive(Clone, Debug)]
pub enum Item {
    Screen(ScreenDef),
    Component(ComponentDef),
    Import(ImportDef),
    Provider(ProviderDef),
}

/// Provider scope — mirrors Riverpod ProviderScope / autoDispose / family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderScope {
    Screen,
    App,
    Family,
}

#[derive(Clone, Debug)]
pub struct ProviderDef {
    pub name: Symbol,
    pub scope: ProviderScope,
    pub family_key: Option<Symbol>,
    pub params: Vec<Param>,
    pub states: Vec<StateDef>,
    pub actions: Vec<ActionDef>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ActionDef {
    pub name: Symbol,
    pub params: Vec<Param>,
    pub is_async: bool,
    pub handler: HandlerId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct InjectDef {
    pub name: Symbol,
    pub ty: TypeRef,
    pub family_args: Vec<ExprId>,
    pub span: Span,
}

/// Side-effect listener — like Riverpod `ref.listen`.
#[derive(Clone, Debug)]
pub struct ListenDef {
    pub expr: ExprId,
    pub binding: Symbol,
    pub handler: HandlerId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ScreenDef {
    pub name: Symbol,
    pub params: Vec<Param>,
    pub injects: Vec<InjectDef>,
    pub listens: Vec<ListenDef>,
    pub state: Vec<StateDef>,
    pub lifecycle: Vec<LifecycleDef>,
    pub body: Vec<ScreenBodyItem>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ScreenBodyItem {
    Node(NodeId),
    Match(MatchDef),
}

#[derive(Clone, Debug)]
pub struct MatchDef {
    pub expr: ExprId,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Vec<NodeId>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum MatchPattern {
    Ident(Symbol),
    Call { name: Symbol, binding: Option<Symbol> },
    Wildcard,
}

#[derive(Clone, Debug)]
pub struct ComponentDef {
    pub name: Symbol,
    pub params: Vec<Param>,
    pub body: Vec<NodeId>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ImportDef {
    pub path: Vec<Symbol>,
    pub items: Vec<Symbol>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: Symbol,
    pub ty: TypeRef,
    pub default: Option<ExprId>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StateDef {
    pub name: Symbol,
    pub ty: TypeRef,
    pub init: ExprId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct LifecycleDef {
    pub kind: LifecycleKind,
    pub handler: HandlerId,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LifecycleKind {
    OnLoad,
    OnAppear,
    OnDispose,
}

#[derive(Clone, Debug)]
pub struct Handler {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Assign {
        target: LValue,
        op: AssignOp,
        value: ExprId,
        span: Span,
    },
    IncDec {
        target: LValue,
        op: IncDecOp,
        span: Span,
    },
    Call {
        callee: ExprId,
        span: Span,
    },
    If {
        cond: ExprId,
        then_: Vec<Stmt>,
        else_: Option<Vec<Stmt>>,
        span: Span,
    },
    Await {
        expr: ExprId,
        span: Span,
    },
}

#[derive(Clone, Debug)]
pub struct LValue {
    pub path: Vec<Symbol>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignOp {
    Set,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IncDecOp {
    Inc,
    Dec,
}

#[derive(Clone, Debug)]
pub enum AstNode {
    Element {
        kind: Symbol,
        args: Vec<Arg>,
        modifiers: Vec<Modifier>,
        children: Vec<NodeId>,
        handler: Option<HandlerId>,
        span: Span,
    },
    If {
        cond: ExprId,
        then_: Vec<NodeId>,
        else_: Option<Vec<NodeId>>,
        span: Span,
    },
    List {
        source: ExprId,
        key: Option<ExprId>,
        binding: Symbol,
        body: Vec<NodeId>,
        span: Span,
    },
    Error,
}

#[derive(Clone, Debug)]
pub struct Arg {
    pub name: Option<Symbol>,
    pub value: ExprId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Modifier {
    pub name: Symbol,
    pub args: Vec<Arg>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(Symbol),
    Interp(Vec<InterpPart>),
    Ident(Symbol),
    Field {
        base: ExprId,
        field: Symbol,
        span: Span,
    },
    Call {
        callee: ExprId,
        args: Vec<Arg>,
        span: Span,
    },
    Await(ExprId, Span),
    Unary {
        op: UnOp,
        rhs: ExprId,
        span: Span,
    },
    Binary {
        op: BinOp,
        lhs: ExprId,
        rhs: ExprId,
        span: Span,
    },
    Error,
}

#[derive(Clone, Debug)]
pub enum InterpPart {
    Text(Symbol),
    Expr(ExprId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnOp {
    Not,
    Neg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Or,
    And,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl BinOp {
    pub fn precedence(&self) -> (u8, u8) {
        match self {
            BinOp::Or => (1, 2),
            BinOp::And => (3, 4),
            BinOp::Eq | BinOp::NotEq => (5, 6),
            BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => (7, 8),
            BinOp::Add | BinOp::Sub => (9, 10),
            BinOp::Mul | BinOp::Div | BinOp::Mod => (11, 12),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TypeRef {
    pub name: Symbol,
    pub generics: Vec<TypeRef>,
    pub optional: bool,
    pub span: Span,
}

impl TypeRef {
    pub fn simple(name: Symbol, span: Span) -> Self {
        Self {
            name,
            generics: Vec::new(),
            optional: false,
            span,
        }
    }
}
