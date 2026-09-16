//! Native renderer — builds command buffers for iOS UIKit / Android View hosts.

use std::collections::HashMap;

use flash_ir::{NodeId, NodeKind};

use crate::host::HostCallbacks;
use crate::renderer::{CommandBuffer, CommandOp, PlatformRenderer, PropKey, PropValue};

/// Renderer that batches ops and flushes to a native host on `commit()`.
pub struct NativeRenderer {
    buffer: CommandBuffer,
    next_handle: u32,
    node_handles: HashMap<u32, u32>,
    host: Box<dyn HostCallbacks>,
}

impl NativeRenderer {
    pub fn new(host: Box<dyn HostCallbacks>) -> Self {
        Self {
            buffer: CommandBuffer::new(),
            next_handle: 1,
            node_handles: HashMap::new(),
            host,
        }
    }

    pub fn pending_ops(&self) -> usize {
        self.buffer.len()
    }

    pub fn encode_pending(&self) -> Vec<u8> {
        self.buffer.encode()
    }

    pub fn register_node(&mut self, node: NodeId, kind: NodeKind) -> u32 {
        let handle = self.next_handle;
        self.next_handle += 1;
        self.node_handles.insert(node.0, handle);
        self.buffer.ops.push(CommandOp::Create { kind, handle });
        handle
    }

    pub fn handle_for(&self, node: NodeId) -> Option<u32> {
        self.node_handles.get(&node.0).copied()
    }
}

impl PlatformRenderer for NativeRenderer {
    type Handle = u32;

    fn create(&mut self, kind: NodeKind) -> Self::Handle {
        let handle = self.next_handle;
        self.next_handle += 1;
        self.buffer.ops.push(CommandOp::Create { kind, handle });
        handle
    }

    fn set_prop(&mut self, handle: Self::Handle, key: PropKey, value: PropValue) {
        self.buffer
            .ops
            .push(CommandOp::SetProp { handle, key, value });
    }

    fn insert_child(&mut self, parent: Self::Handle, child: Self::Handle, index: u32) {
        self.buffer.ops.push(CommandOp::InsertChild {
            parent,
            child,
            index,
        });
    }

    fn remove(&mut self, handle: Self::Handle) {
        self.buffer.ops.push(CommandOp::Remove { handle });
    }

    fn set_handler(&mut self, handle: Self::Handle, handler_id: u32) {
        self.buffer
            .ops
            .push(CommandOp::SetHandler { handle, handler_id });
    }

    fn commit(&mut self) {
        if !self.buffer.ops.is_empty() {
            let bytes = self.buffer.encode();
            self.host.apply_ops(&bytes);
            self.buffer.clear();
        }
    }
}

/// Set a prop on a Flash `NodeId` (maps to native handle).
pub fn set_node_prop(
    renderer: &mut NativeRenderer,
    node: NodeId,
    key: PropKey,
    value: PropValue,
) {
    if let Some(handle) = renderer.handle_for(node) {
        renderer.set_prop(handle, key, value);
    }
}
