//! Rust flex layout engine — single pass, no Auto Layout / double-measure.
//! Phase 5b: replaces UIStackView and nested LinearLayout for deep trees.

use flash_ir::layout::{Align, FlexDirection, FrameCommand, LayoutSpec};

pub struct LayoutEngine {
    specs: Vec<LayoutSpec>,
}

impl LayoutEngine {
    pub fn new(specs: Vec<LayoutSpec>) -> Self {
        Self { specs }
    }

    /// Single top-down flex pass. Returns frame commands batched into command buffer.
    pub fn compute_frames(&self, viewport_width: f32, viewport_height: f32) -> Vec<FrameCommand> {
        let mut frames = Vec::new();
        // MVP: placeholder for Phase 5b implementation
        if let Some(root) = self.specs.first() {
            frames.push(FrameCommand {
                node: root.node,
                x: 0.0,
                y: 0.0,
                width: viewport_width,
                height: viewport_height,
            });
        }
        frames
    }
}

pub fn direction_from_node_kind(name: &str) -> Option<FlexDirection> {
    match name {
        "Row" => Some(FlexDirection::Row),
        "Column" => Some(FlexDirection::Column),
        _ => None,
    }
}
