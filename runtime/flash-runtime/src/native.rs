//! Native rendering — mount UI tree via `NativeRenderer` → iOS UIKit / Android View.

use flash_ir::{HandlerId, IrExpr, NodeId, PropKey, ScreenIr, UpdateOp};
use flash_platform::{
    host::ir_prop_to_platform, native_renderer::set_node_prop, HostCallbacks, NativeRenderer,
    PlatformRenderer, PropValue,
};

use crate::reactive::ReactiveEngine;
use crate::renderer::PropValue as RuntimePropValue;

/// Mount screen IR to a native host (UIKit / Android View / in-process test host).
pub fn mount_screen(engine: &ReactiveEngine, renderer: &mut NativeRenderer) {
    build_tree(engine, renderer);
    write_props(engine, renderer);
    for (node_idx, node) in engine.screen.nodes.iter().enumerate() {
        if let Some(handler) = node.handler {
            if let Some(handle) = renderer.handle_for(NodeId(node_idx as u32)) {
                renderer.set_handler(handle, handler.0);
            }
        }
    }
    renderer.commit();
}

/// Flush dirty slots to native props only (fine-grained update).
pub fn flush_screen(engine: &mut ReactiveEngine, renderer: &mut NativeRenderer) {
    let dirty = engine.slots.drain_dirty();
    for slot in dirty {
        for &op_idx in engine.screen.deps.ops_for(slot) {
            let op = &engine.screen.update_ops[op_idx as usize];
            let value = eval_update_op(engine, op);
            set_node_prop(
                renderer,
                op.node,
                ir_prop_to_platform(op.key),
                runtime_to_platform(value),
            );
        }
    }
    renderer.commit();
}

fn build_tree(engine: &ReactiveEngine, renderer: &mut NativeRenderer) {
    for (idx, node) in engine.screen.nodes.iter().enumerate() {
        renderer.register_node(NodeId(idx as u32), node.kind);
    }
    for (idx, node) in engine.screen.nodes.iter().enumerate() {
        let child_id = NodeId(idx as u32);
        if let Some(parent) = node.parent {
            let parent_node = &engine.screen.nodes[parent.0 as usize];
            let index = parent_node
                .children
                .iter()
                .position(|c| *c == child_id)
                .unwrap_or(0) as u32;
            if let (Some(ph), Some(ch)) = (
                renderer.handle_for(parent),
                renderer.handle_for(child_id),
            ) {
                renderer.insert_child(ph, ch, index);
            }
        }
    }
}

fn write_props(engine: &ReactiveEngine, renderer: &mut NativeRenderer) {
    for prop in &engine.screen.static_props {
        set_node_prop(
            renderer,
            prop.node,
            ir_prop_to_platform(prop.key),
            static_prop_value(&prop.value),
        );
    }
    for op in &engine.screen.update_ops {
        let value = eval_update_op(engine, op);
        set_node_prop(
            renderer,
            op.node,
            ir_prop_to_platform(op.key),
            runtime_to_platform(value),
        );
    }
}

fn eval_update_op(engine: &ReactiveEngine, op: &UpdateOp) -> RuntimePropValue {
    let text = engine
        .screen
        .exprs
        .get(op.expr.0 as usize)
        .map(|e| eval_ir_expr(engine, e))
        .unwrap_or_default();
    match op.key {
        PropKey::Text | PropKey::Title => RuntimePropValue::Str(text),
        _ => RuntimePropValue::Str(text),
    }
}

fn eval_ir_expr(engine: &ReactiveEngine, expr: &IrExpr) -> String {
    match expr {
        IrExpr::Int(v) => v.to_string(),
        IrExpr::Float(v) => v.to_string(),
        IrExpr::Bool(v) => v.to_string(),
        IrExpr::Str(s) => s.clone(),
        IrExpr::Slot(slot) => engine.slots.get_int(*slot).to_string(),
        IrExpr::Concat(parts) => parts.iter().map(|p| eval_ir_expr(engine, p)).collect(),
        IrExpr::Add(slot, v) => (engine.slots.get_int(*slot) + *v).to_string(),
        IrExpr::Error => String::new(),
    }
}

fn static_prop_value(expr: &IrExpr) -> PropValue {
    match expr {
        IrExpr::Str(s) => PropValue::Str(s.clone()),
        IrExpr::Int(v) => PropValue::Str(v.to_string()),
        _ => PropValue::Str(String::new()),
    }
}

fn runtime_to_platform(value: RuntimePropValue) -> PropValue {
    match value {
        RuntimePropValue::Str(s) => PropValue::Str(s),
        RuntimePropValue::Int(v) => PropValue::Int(v),
        RuntimePropValue::Float(v) => PropValue::Float(v),
        RuntimePropValue::Bool(v) => PropValue::Bool(v),
    }
}

/// Session wrapping engine + native renderer for CLI and tests.
pub struct NativeSession {
    pub engine: ReactiveEngine,
    pub renderer: NativeRenderer,
}

impl NativeSession {
    pub fn mount(screen: ScreenIr, host: Box<dyn HostCallbacks>) -> Self {
        let engine = ReactiveEngine::new(screen);
        let mut renderer = NativeRenderer::new(host);
        mount_screen(&engine, &mut renderer);
        Self { engine, renderer }
    }

    pub fn fire_handler(&mut self, handler_id: HandlerId) {
        self.engine.fire_handler(handler_id);
        flush_screen(&mut self.engine, &mut self.renderer);
    }
}
