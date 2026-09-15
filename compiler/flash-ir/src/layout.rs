//! Layout IR — single-pass flex layout, bypassing Auto Layout / double-measure.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Clone, Debug)]
pub struct LayoutSpec {
    pub node: super::NodeId,
    pub direction: FlexDirection,
    pub gap: f32,
    pub padding: f32,
    pub align: Align,
    pub child_weight: Option<f32>,
}

/// Frame assignment is batched separately from reactive property updates.
#[derive(Clone, Debug)]
pub struct FrameCommand {
    pub node: super::NodeId,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
