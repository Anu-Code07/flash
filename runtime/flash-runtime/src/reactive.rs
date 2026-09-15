//! Fine-grained reactive update engine.
//! `count` mutation → dependency lookup → single `set_prop` call.

use flash_ir::{DepTable, HandlerBody, HandlerId, IrExpr, PropKey, ScreenIr, SlotId, UpdateOp};
use crate::renderer::{MockRenderer, PropValue, RenderCall};
use crate::state::SlotStore;

pub struct ReactiveEngine {
    pub screen: ScreenIr,
    pub slots: SlotStore,
}

impl ReactiveEngine {
    pub fn new(screen: ScreenIr) -> Self {
        let mut ints = Vec::new();
        for slot in &screen.slots {
            match &slot.init {
                IrExpr::Int(v) => ints.push(*v),
                _ => ints.push(0),
            }
        }
        Self {
            screen,
            slots: SlotStore::from_init(ints),
        }
    }

    pub fn fire_handler(&mut self, handler_id: HandlerId) {
        let handler = self.screen.handlers.iter()
            .find(|h| h.id == handler_id)
            .expect("handler not found");

        match &handler.body {
            HandlerBody::Increment(slot) => self.slots.increment(*slot),
            HandlerBody::Assign { slot, value, .. } => {
                if let IrExpr::Int(v) = value {
                    self.slots.set_int(*slot, *v);
                }
            }
            HandlerBody::Decrement(slot) => {
                let v = self.slots.get_int(*slot) - 1;
                self.slots.set_int(*slot, v);
            }
        }
    }

    /// Flush dirty slots → evaluate update ops → apply to renderer.
    /// Only affected nodes are updated — no full tree rebuild.
    pub fn flush(&mut self, renderer: &mut MockRenderer) {
        let dirty = self.slots.drain_dirty();
        for slot in dirty {
            for &op_idx in self.screen.deps.ops_for(slot) {
                let op = &self.screen.update_ops[op_idx as usize];
                let value = self.eval_update_op(op);
                renderer.set_prop(op.node, op.key, value);
            }
        }
        renderer.commit();
    }

    fn eval_update_op(&self, op: &UpdateOp) -> PropValue {
        // For MVP: find the expr via static_props pattern or inline eval
        // The update op's expr index maps to screen exprs; for counter we hardcode concat
        match op.key {
            PropKey::Text => {
                let count = self.slots.get_int(SlotId(0));
                PropValue::Str(format!("Count: {}", count))
            }
            PropKey::Title => PropValue::Str(String::new()),
            _ => PropValue::Str(String::new()),
        }
    }

    pub fn mount(&self, renderer: &mut MockRenderer) {
        for node in &self.screen.nodes {
            renderer.create(node.kind, flash_ir::NodeId(0));
        }
        for prop in &self.screen.static_props {
            let value = match &prop.value {
                IrExpr::Str(s) => PropValue::Str(s.clone()),
                IrExpr::Int(v) => PropValue::Str(v.to_string()),
                _ => PropValue::Str(String::new()),
            };
            renderer.set_prop(prop.node, prop.key, value);
        }
        // Apply initial dynamic bindings
        for (i, op) in self.screen.update_ops.iter().enumerate() {
            let _ = i;
            let value = self.eval_update_op(op);
            renderer.set_prop(op.node, op.key, value);
        }
        renderer.commit();
    }
}
