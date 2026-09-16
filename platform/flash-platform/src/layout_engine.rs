//! Rust flex layout engine — single pass, assigns frames to Column/Row/Stack children.

use flash_ir::layout::{FlexDirection, FrameCommand};
use flash_ir::{NodeId, NodeIr, NodeKind, ScreenIr};

const DEFAULT_PADDING: f32 = 16.0;
const DEFAULT_GAP: f32 = 8.0;
const VIEWPORT_W: f32 = 390.0;
const VIEWPORT_H: f32 = 844.0;

struct Measured {
    width: f32,
    height: f32,
}

pub struct LayoutEngine;

impl LayoutEngine {
    /// Compute frame commands for all nodes in a screen.
    pub fn layout_screen(screen: &ScreenIr) -> Vec<FrameCommand> {
        let root = screen.nodes.iter().position(|n| n.parent.is_none());
        let Some(root_idx) = root else {
            return Vec::new();
        };
        let measured = measure_node(screen, NodeId(root_idx as u32), VIEWPORT_W, VIEWPORT_H);
        let mut frames = Vec::new();
        assign_frames(
            screen,
            NodeId(root_idx as u32),
            0.0,
            0.0,
            measured.width,
            measured.height,
            &mut frames,
        );
        frames
    }
}

fn measure_node(screen: &ScreenIr, node: NodeId, max_w: f32, max_h: f32) -> Measured {
    let n = &screen.nodes[node.0 as usize];
    match n.kind {
        NodeKind::COLUMN => measure_flex(screen, n, FlexDirection::Column, max_w, max_h),
        NodeKind::ROW => measure_flex(screen, n, FlexDirection::Row, max_w, max_h),
        NodeKind::STACK => measure_stack(screen, n, max_w, max_h),
        _ => Measured {
            width: intrinsic_width(n.kind),
            height: intrinsic_height(n.kind),
        },
    }
}

fn measure_flex(
    screen: &ScreenIr,
    node: &NodeIr,
    dir: FlexDirection,
    max_w: f32,
    max_h: f32,
) -> Measured {
    let mut children_size = 0.0f32;
    let mut cross_max = 0.0f32;
    let gap = DEFAULT_GAP;
    let pad = DEFAULT_PADDING;

    for &child_id in &node.children {
        let child = measure_node(screen, child_id, max_w - pad * 2.0, max_h - pad * 2.0);
        match dir {
            FlexDirection::Column => {
                children_size += child.height;
                cross_max = cross_max.max(child.width);
            }
            FlexDirection::Row => {
                children_size += child.width;
                cross_max = cross_max.max(child.height);
            }
        }
    }
    if node.children.len() > 1 {
        children_size += gap * (node.children.len() - 1) as f32;
    }

    match dir {
        FlexDirection::Column => Measured {
            width: (cross_max + pad * 2.0).min(max_w),
            height: (children_size + pad * 2.0).min(max_h),
        },
        FlexDirection::Row => Measured {
            width: (children_size + pad * 2.0).min(max_w),
            height: (cross_max + pad * 2.0).min(max_h),
        },
    }
}

fn measure_stack(screen: &ScreenIr, node: &NodeIr, max_w: f32, max_h: f32) -> Measured {
    let mut w = 0.0f32;
    let mut h = 0.0f32;
    for &child_id in &node.children {
        let child = measure_node(screen, child_id, max_w, max_h);
        w = w.max(child.width);
        h = h.max(child.height);
    }
    Measured {
        width: w.max(100.0).min(max_w),
        height: h.max(100.0).min(max_h),
    }
}

fn assign_frames(
    screen: &ScreenIr,
    node: NodeId,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    out: &mut Vec<FrameCommand>,
) {
    out.push(FrameCommand {
        node,
        x,
        y,
        width,
        height,
    });

    let n = &screen.nodes[node.0 as usize];
    let pad = DEFAULT_PADDING;
    let gap = DEFAULT_GAP;
    let inner_x = x + pad;
    let inner_y = y + pad;
    let inner_w = width - pad * 2.0;
    let inner_h = height - pad * 2.0;

    match n.kind {
        NodeKind::COLUMN => {
            let mut cursor = inner_y;
            for &child_id in &n.children {
                let child_m = measure_node(screen, child_id, inner_w, inner_h);
                assign_frames(screen, child_id, inner_x, cursor, inner_w, child_m.height, out);
                cursor += child_m.height + gap;
            }
        }
        NodeKind::ROW => {
            let mut cursor = inner_x;
            for &child_id in &n.children {
                let child_m = measure_node(screen, child_id, inner_w, inner_h);
                assign_frames(screen, child_id, cursor, inner_y, child_m.width, inner_h, out);
                cursor += child_m.width + gap;
            }
        }
        NodeKind::STACK => {
            for &child_id in &n.children {
                assign_frames(screen, child_id, inner_x, inner_y, inner_w, inner_h, out);
            }
        }
        _ => {}
    }
}

fn intrinsic_width(kind: NodeKind) -> f32 {
    match kind.name() {
        "Text" => 200.0,
        "Button" => 120.0,
        "TextField" => 280.0,
        "Image" => 200.0,
        "Loading" => 40.0,
        _ => 100.0,
    }
}

fn intrinsic_height(kind: NodeKind) -> f32 {
    match kind.name() {
        "Text" => 22.0,
        "Button" => 44.0,
        "TextField" => 44.0,
        "Image" => 200.0,
        "Loading" => 40.0,
        _ => 40.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flash_ir::{IrExpr, NodeIr, PropKey, SlotDef, SlotId, StaticProp, UpdateOp};
    use flash_span::Symbol;
    use flash_stl::types::TypeKind;

    fn counter_screen() -> ScreenIr {
        ScreenIr {
            name: Symbol(0),
            slots: vec![SlotDef {
                name: Symbol(1),
                ty: TypeKind::Int,
                init: IrExpr::Int(0),
            }],
            nodes: vec![
                NodeIr {
                    kind: NodeKind::COLUMN,
                    parent: None,
                    children: vec![NodeId(1), NodeId(2)],
                    handler: None,
                },
                NodeIr {
                    kind: NodeKind::TEXT,
                    parent: Some(NodeId(0)),
                    children: vec![],
                    handler: None,
                },
                NodeIr {
                    kind: NodeKind::BUTTON,
                    parent: Some(NodeId(0)),
                    children: vec![],
                    handler: Some(flash_ir::HandlerId(0)),
                },
            ],
            static_props: vec![StaticProp {
                node: NodeId(2),
                key: PropKey::Title,
                value: IrExpr::Str("Go".into()),
            }],
            update_ops: vec![UpdateOp {
                node: NodeId(1),
                key: PropKey::Text,
                expr: flash_ir::ExprId(0),
                reads: vec![SlotId(0)],
            }],
            deps: flash_ir::DepTable::default(),
            handlers: vec![],
            actions: vec![],
            listens: vec![],
            exprs: vec![IrExpr::Slot(SlotId(0))],
        }
    }

    #[test]
    fn layout_assigns_frames_to_all_nodes() {
        let frames = LayoutEngine::layout_screen(&counter_screen());
        assert_eq!(frames.len(), 3);
        assert!(frames[0].width > 0.0);
        assert!(frames[1].height >= 22.0);
    }
}
