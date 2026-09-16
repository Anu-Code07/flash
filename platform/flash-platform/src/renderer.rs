//! Platform-agnostic renderer trait and command buffer.
//! Mobile targets batch all property updates into one FFI call per frame.

use flash_ir::NodeKind as IrNodeKind;

/// Property keys shared across all platforms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PropKey {
    Text,
    Title,
    Value,
    Src,
    FontSize,
    FontWeight,
    Color,
    Padding,
    Background,
}

/// Platform-neutral property value.
#[derive(Clone, Debug, PartialEq)]
pub enum PropValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

/// Maps IR node kinds to platform renderer node kinds.
pub type NodeKind = IrNodeKind;

/// Command buffer: encodes batched property updates for one FFI crossing per frame.
/// Critical on Android where JNI call overhead is high.
#[derive(Clone, Debug, Default)]
pub struct CommandBuffer {
    pub ops: Vec<CommandOp>,
}

#[derive(Clone, Debug)]
pub enum CommandOp {
    Create { kind: NodeKind, handle: u32 },
    SetProp { handle: u32, key: PropKey, value: PropValue },
    InsertChild { parent: u32, child: u32, index: u32 },
    Remove { handle: u32 },
    SetHandler { handle: u32, handler_id: u32 },
    SetFrame { handle: u32, x: f32, y: f32, width: f32, height: f32 },
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.ops.clear();
    }

    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Encode buffer to bytes for cross-boundary transfer.
    pub fn encode(&self) -> Vec<u8> {
        // MVP: simple binary encoding
        let mut buf = Vec::new();
        for op in &self.ops {
            match op {
                CommandOp::Create { kind, handle } => {
                    buf.push(0);
                    buf.extend_from_slice(&handle.to_le_bytes());
                    buf.extend_from_slice(&kind.as_u16().to_le_bytes());
                }
                CommandOp::SetProp { handle, key, value } => {
                    buf.push(1);
                    buf.extend_from_slice(&handle.to_le_bytes());
                    buf.extend_from_slice(&(key_as_u16(key)).to_le_bytes());
                    encode_value(&mut buf, value);
                }
                CommandOp::InsertChild { parent, child, index } => {
                    buf.push(2);
                    buf.extend_from_slice(&parent.to_le_bytes());
                    buf.extend_from_slice(&child.to_le_bytes());
                    buf.extend_from_slice(&index.to_le_bytes());
                }
                CommandOp::Remove { handle } => {
                    buf.push(3);
                    buf.extend_from_slice(&handle.to_le_bytes());
                }
                CommandOp::SetHandler { handle, handler_id } => {
                    buf.push(4);
                    buf.extend_from_slice(&handle.to_le_bytes());
                    buf.extend_from_slice(&handler_id.to_le_bytes());
                }
                CommandOp::SetFrame {
                    handle,
                    x,
                    y,
                    width,
                    height,
                } => {
                    buf.push(5);
                    buf.extend_from_slice(&handle.to_le_bytes());
                    buf.extend_from_slice(&x.to_le_bytes());
                    buf.extend_from_slice(&y.to_le_bytes());
                    buf.extend_from_slice(&width.to_le_bytes());
                    buf.extend_from_slice(&height.to_le_bytes());
                }
            }
        }
        buf
    }
}

fn key_as_u16(key: &PropKey) -> u16 {
    match key {
        PropKey::Text => 0,
        PropKey::Title => 1,
        PropKey::Value => 2,
        PropKey::Src => 3,
        PropKey::FontSize => 4,
        PropKey::FontWeight => 5,
        PropKey::Color => 6,
        PropKey::Padding => 7,
        PropKey::Background => 8,
    }
}

fn encode_value(buf: &mut Vec<u8>, value: &PropValue) {
    match value {
        PropValue::Str(s) => {
            buf.push(0);
            let bytes = s.as_bytes();
            buf.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(bytes);
        }
        PropValue::Int(v) => {
            buf.push(1);
            buf.extend_from_slice(&v.to_le_bytes());
        }
        PropValue::Float(v) => {
            buf.push(2);
            buf.extend_from_slice(&v.to_le_bytes());
        }
        PropValue::Bool(v) => {
            buf.push(3);
            buf.push(if *v { 1 } else { 0 });
        }
    }
}

/// The renderer seam — implemented by iOS, Android, and Web adapters.
pub trait PlatformRenderer {
    type Handle: Copy + Eq + std::fmt::Debug;

    fn create(&mut self, kind: NodeKind) -> Self::Handle;
    fn set_prop(&mut self, handle: Self::Handle, key: PropKey, value: PropValue);
    fn insert_child(&mut self, parent: Self::Handle, child: Self::Handle, index: u32);
    fn remove(&mut self, handle: Self::Handle);
    fn set_handler(&mut self, handle: Self::Handle, handler_id: u32);
    fn set_frame(&mut self, handle: Self::Handle, x: f32, y: f32, width: f32, height: f32);
    /// One boundary crossing per frame — flushes the command buffer to native UI.
    fn commit(&mut self);
}
