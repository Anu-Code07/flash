//! Fine-grained reactive update engine.

use flash_ir::{HandlerBody, HandlerId, IrExpr, PropKey, ScreenIr, UpdateOp};
use crate::renderer::{MockRenderer, PropValue};
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
        let body = self.screen.handlers.iter()
            .find(|h| h.id == handler_id)
            .expect("handler not found")
            .body
            .clone();
        self.execute_handler_body(&body);
    }

    fn execute_handler_body(&mut self, body: &HandlerBody) {
        match body {
            HandlerBody::Increment(slot) => self.slots.increment(*slot),
            HandlerBody::Decrement(slot) => {
                let v = self.slots.get_int(*slot) - 1;
                self.slots.set_int(*slot, v);
            }
            HandlerBody::Assign { slot, value, .. } => {
                if let IrExpr::Int(v) = value {
                    self.slots.set_int(*slot, *v);
                }
            }
            HandlerBody::InvokeAction(idx) => {
                let ops = self.screen.actions
                    .get(*idx as usize)
                    .map(|a| a.body.clone())
                    .unwrap_or_default();
                for op in ops {
                    self.execute_handler_body(&op);
                }
            }
            HandlerBody::Sequence(ops) => {
                for op in ops {
                    self.execute_handler_body(op);
                }
            }
        }
    }

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
        let expr = self.screen.exprs.get(op.expr.0 as usize);
        let text = expr
            .map(|e| self.eval_ir_expr(e))
            .unwrap_or_default();
        match op.key {
            PropKey::Text | PropKey::Title => PropValue::Str(text),
            _ => PropValue::Str(text),
        }
    }

    fn eval_ir_expr(&self, expr: &IrExpr) -> String {
        match expr {
            IrExpr::Int(v) => v.to_string(),
            IrExpr::Float(v) => v.to_string(),
            IrExpr::Bool(v) => v.to_string(),
            IrExpr::Str(s) => s.clone(),
            IrExpr::Slot(slot) => self.slots.get_int(*slot).to_string(),
            IrExpr::Concat(parts) => parts.iter().map(|p| self.eval_ir_expr(p)).collect(),
            IrExpr::Add(slot, v) => (self.slots.get_int(*slot) + *v).to_string(),
            IrExpr::Error => String::new(),
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
        for op in &self.screen.update_ops {
            let value = self.eval_update_op(op);
            renderer.set_prop(op.node, op.key, value);
        }
        renderer.commit();
    }
}
