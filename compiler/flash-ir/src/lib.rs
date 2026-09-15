//! UI Intermediate Representation — flat, optimized for incremental mobile updates.

pub mod animation;
pub mod layout;

use flash_span::Symbol;
use flash_stl::types::TypeKind;
use flash_stl::widgets::{widget_by_id, widget_by_name, WidgetId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlotId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OpId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HandlerId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExprId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RegionId(pub u32);

/// Top-level compiled UI IR for a module.
#[derive(Clone, Debug)]
pub struct UiIr {
    pub screens: Vec<ScreenIr>,
    pub components: Vec<ComponentIr>,
    pub providers: Vec<ProviderIr>,
}

#[derive(Clone, Debug)]
pub struct ScreenIr {
    pub name: Symbol,
    pub slots: Vec<SlotDef>,
    pub nodes: Vec<NodeIr>,
    pub static_props: Vec<StaticProp>,
    pub update_ops: Vec<UpdateOp>,
    pub deps: DepTable,
    pub handlers: Vec<HandlerIr>,
    pub actions: Vec<CompiledAction>,
    pub listens: Vec<ListenIr>,
    pub exprs: Vec<IrExpr>,
}

#[derive(Clone, Debug)]
pub struct ProviderIr {
    pub name: Symbol,
    pub scope: ProviderScope,
    pub state_fields: Vec<Symbol>,
    pub actions: Vec<Symbol>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderScope {
    Screen,
    App,
    Family,
}

#[derive(Clone, Debug)]
pub struct CompiledAction {
    pub name: Symbol,
    pub provider: Option<Symbol>,
    pub writes: Vec<SlotId>,
    pub body: Vec<HandlerBody>,
    pub is_async: bool,
}

#[derive(Clone, Debug)]
pub struct ListenIr {
    pub reads: Vec<SlotId>,
    pub handler: HandlerId,
}

#[derive(Clone, Debug)]
pub struct ComponentIr {
    pub name: Symbol,
    pub params: Vec<ParamDef>,
    pub nodes: Vec<NodeIr>,
    pub static_props: Vec<StaticProp>,
    pub update_ops: Vec<UpdateOp>,
    pub handlers: Vec<HandlerIr>,
    pub exprs: Vec<IrExpr>,
}

#[derive(Clone, Debug)]
pub struct SlotDef {
    pub name: Symbol,
    pub ty: TypeKind,
    pub init: IrExpr,
}

#[derive(Clone, Debug)]
pub struct ParamDef {
    pub name: Symbol,
    pub ty: TypeKind,
}

/// Widget kind in IR — wraps stable `WidgetId` from the STL catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeKind(WidgetId);

impl NodeKind {
    pub const TEXT: Self = Self(WidgetId(0));
    pub const BUTTON: Self = Self(WidgetId(1));
    pub const COLUMN: Self = Self(WidgetId(2));
    pub const ROW: Self = Self(WidgetId(3));
    pub const STACK: Self = Self(WidgetId(4));
    pub const IMAGE: Self = Self(WidgetId(5));
    pub const TEXT_FIELD: Self = Self(WidgetId(6));
    pub const SCROLL_VIEW: Self = Self(WidgetId(7));
    pub const LIST: Self = Self(WidgetId(8));
    pub const LOADING: Self = Self(WidgetId(9));

    pub fn from_name(name: &str) -> Option<Self> {
        widget_by_name(name).map(|w| NodeKind(w.id))
    }

    pub fn name(&self) -> &'static str {
        widget_by_id(self.0).map(|w| w.name).unwrap_or("Unknown")
    }

    pub fn id(&self) -> WidgetId {
        self.0
    }

    pub fn as_u16(&self) -> u16 {
        self.0.0
    }
}

#[derive(Clone, Debug)]
pub struct NodeIr {
    pub kind: NodeKind,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub handler: Option<HandlerId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropKey {
    Text,
    Title,
    Value,
    Src,
}

#[derive(Clone, Debug)]
pub struct StaticProp {
    pub node: NodeId,
    pub key: PropKey,
    pub value: IrExpr,
}

#[derive(Clone, Debug)]
pub struct UpdateOp {
    pub node: NodeId,
    pub key: PropKey,
    pub expr: ExprId,
    pub reads: Vec<SlotId>,
}

#[derive(Clone, Debug)]
pub struct HandlerIr {
    pub id: HandlerId,
    pub node: NodeId,
    pub writes: Vec<SlotId>,
    pub body: HandlerBody,
}

#[derive(Clone, Debug)]
pub enum HandlerBody {
    Increment(SlotId),
    Decrement(SlotId),
    Assign { slot: SlotId, op: AssignOp, value: IrExpr },
    /// Invoke a compiled @action (Riverpod Notifier method).
    InvokeAction(u32),
    /// Multiple ops from an @action body.
    Sequence(Vec<HandlerBody>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignOp {
    Set,
    Add,
}

/// CSR (compressed sparse row) dependency table — O(1) slot → update ops lookup.
#[derive(Clone, Debug, Default)]
pub struct DepTable {
    pub offsets: Vec<u32>,
    pub ops: Vec<u32>,
}

impl DepTable {
    pub fn build(num_slots: usize, update_ops: &[UpdateOp]) -> Self {
        let mut slot_ops: Vec<Vec<u32>> = vec![Vec::new(); num_slots];
        for (i, op) in update_ops.iter().enumerate() {
            for &slot in &op.reads {
                slot_ops[slot.0 as usize].push(i as u32);
            }
        }
        let mut offsets = Vec::with_capacity(num_slots + 1);
        let mut ops = Vec::new();
        let mut cursor = 0u32;
        for slot_list in &slot_ops {
            offsets.push(cursor);
            ops.extend(slot_list);
            cursor += slot_list.len() as u32;
        }
        offsets.push(cursor);
        DepTable { offsets, ops }
    }

    pub fn ops_for(&self, slot: SlotId) -> &[u32] {
        let start = self.offsets[slot.0 as usize] as usize;
        let end = self.offsets[slot.0 as usize + 1] as usize;
        &self.ops[start..end]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum IrExpr {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Slot(SlotId),
    Concat(Vec<IrExpr>),
    Add(SlotId, i64),
    Error,
}

/// Pretty-print IR for `flash ir` CLI command.
pub fn format_screen_ir(screen: &ScreenIr, interner: &flash_span::Interner) -> String {
    let mut out = String::new();
    out.push_str(&format!("SCREEN {}\n", interner.get(screen.name)));

    out.push_str("  SLOTS\n");
    for (i, slot) in screen.slots.iter().enumerate() {
        out.push_str(&format!("    {}  {} : {}\n", i, interner.get(slot.name), slot.ty));
    }

    out.push_str("\n  NODES\n");
    for (i, node) in screen.nodes.iter().enumerate() {
        let parent = node.parent.map(|p| p.0.to_string()).unwrap_or_else(|| "-".into());
        let children = node.children.iter().map(|c| c.0.to_string()).collect::<Vec<_>>().join(",");
        let handler = node.handler.map(|h| format!("handler=h{}", h.0)).unwrap_or_default();
        out.push_str(&format!(
            "    {}  {:12} parent={} children=[{}] {}\n",
            i, node.kind.name(), parent, children, handler
        ));
    }

    out.push_str("\n  STATIC_PROPS\n");
    for prop in &screen.static_props {
        out.push_str(&format!(
            "    node={} prop={:?} value={}\n",
            prop.node.0, prop.key, format_expr(&prop.value)
        ));
    }

    out.push_str("\n  UPDATE_OPS\n");
    for (i, op) in screen.update_ops.iter().enumerate() {
        let reads = op.reads.iter().map(|s| s.0.to_string()).collect::<Vec<_>>().join(",");
        out.push_str(&format!(
            "    op{}  node={} prop={:?} reads=[{}]\n",
            i, op.node.0, op.key, reads
        ));
    }

    out.push_str("\n  HANDLERS\n");
    for h in &screen.handlers {
        let writes = h.writes.iter().map(|s| s.0.to_string()).collect::<Vec<_>>().join(",");
        out.push_str(&format!("    h{}  node={} writes=[{}] {:?}\n", h.id.0, h.node.0, writes, h.body));
    }

    if !screen.actions.is_empty() {
        out.push_str("\n  ACTIONS\n");
        for (i, action) in screen.actions.iter().enumerate() {
            let writes = action.writes.iter().map(|s| s.0.to_string()).collect::<Vec<_>>().join(",");
            out.push_str(&format!(
                "    a{}  {} writes=[{}] async={}\n",
                i,
                interner.get(action.name),
                writes,
                action.is_async
            ));
        }
    }

    if !screen.listens.is_empty() {
        out.push_str("\n  LISTENERS\n");
        for (i, listen) in screen.listens.iter().enumerate() {
            let reads = listen.reads.iter().map(|s| s.0.to_string()).collect::<Vec<_>>().join(",");
            out.push_str(&format!("    l{}  reads=[{}] handler=h{}\n", i, reads, listen.handler.0));
        }
    }

    out.push_str("\n  DEPENDENCIES\n");
    for (i, slot) in screen.slots.iter().enumerate() {
        let ops = screen.deps.ops_for(SlotId(i as u32));
        if !ops.is_empty() {
            let targets = ops.iter().map(|o| format!("op{}", o)).collect::<Vec<_>>().join(", ");
            out.push_str(&format!("    slot{}({}) → [{}]\n", i, interner.get(slot.name), targets));
        }
    }

    out
}

fn format_expr(expr: &IrExpr) -> String {
    match expr {
        IrExpr::Int(v) => v.to_string(),
        IrExpr::Float(v) => v.to_string(),
        IrExpr::Bool(v) => v.to_string(),
        IrExpr::Str(s) => format!("\"{}\"", s),
        IrExpr::Slot(s) => format!("slot{}", s.0),
        IrExpr::Concat(parts) => {
            let inner = parts.iter().map(format_expr).collect::<Vec<_>>().join(" + ");
            format!("concat({})", inner)
        }
        IrExpr::Add(slot, v) => format!("slot{} + {}", slot.0, v),
        IrExpr::Error => "<error>".into(),
    }
}
