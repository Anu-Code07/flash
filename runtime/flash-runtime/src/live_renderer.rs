//! Live renderer — holds UI state and applies hot-reload patches.

use std::collections::HashMap;

use flash_ir::{NodeId, NodeKind, PropKey};

use crate::renderer::PropValue;

#[derive(Clone, Debug, PartialEq)]
pub enum PatchOp {
    SetProp { node: NodeId, key: PropKey, value: PropValue },
    Remount,
}

#[derive(Clone, Debug)]
struct NodeState {
    kind: NodeKind,
    props: HashMap<PropKey, PropValue>,
}

/// In-memory UI tree for dev mode and hot reload simulation.
#[derive(Clone, Debug, Default)]
pub struct LiveRenderer {
    nodes: Vec<NodeState>,
    pub patch_log: Vec<PatchOp>,
}

impl LiveRenderer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.patch_log.clear();
    }

    pub fn create(&mut self, kind: NodeKind, _node: NodeId) {
        self.nodes.push(NodeState {
            kind,
            props: HashMap::new(),
        });
    }

    pub fn set_prop(&mut self, node: NodeId, key: PropKey, value: PropValue) {
        if let Some(n) = self.nodes.get(node.0 as usize) {
            let prev = n.props.get(&key);
            if prev != Some(&value) {
                self.patch_log.push(PatchOp::SetProp {
                    node,
                    key,
                    value: value.clone(),
                });
            }
        }
        if let Some(n) = self.nodes.get_mut(node.0 as usize) {
            n.props.insert(key, value);
        }
    }

    pub fn commit(&mut self) {
        // Native hosts flush command buffer here; live renderer is synchronous.
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn prop(&self, node: NodeId, key: PropKey) -> Option<&PropValue> {
        self.nodes
            .get(node.0 as usize)
            .and_then(|n| n.props.get(&key))
    }

    pub fn describe(&self) -> Vec<String> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(i, n)| {
                let props = n
                    .props
                    .iter()
                    .map(|(k, v)| format!("{}={}", prop_key_name(k), format_value(v)))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("  node{} {} [{}]", i, n.kind.name(), props)
            })
            .collect()
    }
}

fn prop_key_name(key: &PropKey) -> &'static str {
    match key {
        PropKey::Text => "text",
        PropKey::Title => "title",
        PropKey::Value => "value",
        PropKey::Src => "src",
    }
}

fn format_value(v: &PropValue) -> String {
    match v {
        PropValue::Str(s) => format!("\"{}\"", s),
        PropValue::Int(n) => n.to_string(),
        PropValue::Float(f) => f.to_string(),
        PropValue::Bool(b) => b.to_string(),
    }
}
