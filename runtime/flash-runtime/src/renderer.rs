//! MockRenderer for testing — records every call to verify fine-grained updates.

use flash_ir::{NodeId, NodeKind, PropKey};

#[derive(Clone, Debug, PartialEq)]
pub enum PropValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenderCall {
    Create { kind: NodeKind },
    SetProp { node: NodeId, key: PropKey, value: PropValue },
    Commit,
}

pub struct MockRenderer {
    pub log: Vec<RenderCall>,
    next_handle: u32,
}

impl MockRenderer {
    pub fn new() -> Self {
        Self { log: Vec::new(), next_handle: 0 }
    }

    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    pub fn log(&self) -> &[RenderCall] {
        &self.log
    }

    pub fn create(&mut self, kind: NodeKind, _node: NodeId) {
        self.next_handle += 1;
        self.log.push(RenderCall::Create { kind });
    }

    pub fn set_prop(&mut self, node: NodeId, key: PropKey, value: PropValue) {
        self.log.push(RenderCall::SetProp { node, key, value });
    }

    pub fn commit(&mut self) {
        self.log.push(RenderCall::Commit);
    }
}
