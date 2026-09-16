//! Native host bridge — C vtable + command buffer decode for iOS/Android.

use crate::renderer::PropKey;

/// Platform callbacks invoked when `NativeRenderer::commit()` flushes a frame.
pub trait HostCallbacks: Send {
    fn apply_ops(&mut self, ops: &[u8]);
}

/// Decoded op for native hosts (Swift/Kotlin mirror this format).
#[derive(Clone, Debug, PartialEq)]
pub enum DecodedOp {
    Create { handle: u32, kind: u16 },
    SetProp { handle: u32, key: u16, value: DecodedValue },
    InsertChild { parent: u32, child: u32, index: u32 },
    Remove { handle: u32 },
    SetHandler { handle: u32, handler_id: u32 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum DecodedValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

/// Decode a command buffer produced by `CommandBuffer::encode`.
pub fn decode_ops(bytes: &[u8]) -> Result<Vec<DecodedOp>, DecodeError> {
    let mut ops = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let tag = bytes[i];
        i += 1;
        match tag {
            0 => {
                let (handle, ni) = read_u32(bytes, i)?;
                let (kind, nk) = read_u16(bytes, ni)?;
                i = nk;
                ops.push(DecodedOp::Create { handle, kind });
            }
            1 => {
                let (handle, ni) = read_u32(bytes, i)?;
                let (key, nk) = read_u16(bytes, ni)?;
                let (value, nv) = decode_value(bytes, nk)?;
                i = nv;
                ops.push(DecodedOp::SetProp { handle, key, value });
            }
            2 => {
                let (parent, ni) = read_u32(bytes, i)?;
                let (child, nc) = read_u32(bytes, ni)?;
                let (index, nx) = read_u32(bytes, nc)?;
                i = nx;
                ops.push(DecodedOp::InsertChild { parent, child, index });
            }
            3 => {
                let (handle, ni) = read_u32(bytes, i)?;
                i = ni;
                ops.push(DecodedOp::Remove { handle });
            }
            4 => {
                let (handle, ni) = read_u32(bytes, i)?;
                let (handler_id, nh) = read_u32(bytes, ni)?;
                i = nh;
                ops.push(DecodedOp::SetHandler { handle, handler_id });
            }
            _ => return Err(DecodeError::UnknownOp(tag)),
        }
    }
    Ok(ops)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    UnexpectedEof,
    UnknownOp(u8),
    UnknownValueTag(u8),
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::UnexpectedEof => write!(f, "unexpected end of command buffer"),
            DecodeError::UnknownOp(t) => write!(f, "unknown op tag {}", t),
            DecodeError::UnknownValueTag(t) => write!(f, "unknown value tag {}", t),
        }
    }
}

impl std::error::Error for DecodeError {}

fn read_u32(bytes: &[u8], i: usize) -> Result<(u32, usize), DecodeError> {
    if i + 4 > bytes.len() {
        return Err(DecodeError::UnexpectedEof);
    }
    let v = u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
    Ok((v, i + 4))
}

fn read_u16(bytes: &[u8], i: usize) -> Result<(u16, usize), DecodeError> {
    if i + 2 > bytes.len() {
        return Err(DecodeError::UnexpectedEof);
    }
    let v = u16::from_le_bytes(bytes[i..i + 2].try_into().unwrap());
    Ok((v, i + 2))
}

fn read_i64(bytes: &[u8], i: usize) -> Result<(i64, usize), DecodeError> {
    if i + 8 > bytes.len() {
        return Err(DecodeError::UnexpectedEof);
    }
    let v = i64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    Ok((v, i + 8))
}

fn read_f64(bytes: &[u8], i: usize) -> Result<(f64, usize), DecodeError> {
    if i + 8 > bytes.len() {
        return Err(DecodeError::UnexpectedEof);
    }
    let v = f64::from_le_bytes(bytes[i..i + 8].try_into().unwrap());
    Ok((v, i + 8))
}

fn decode_value(bytes: &[u8], i: usize) -> Result<(DecodedValue, usize), DecodeError> {
    if i >= bytes.len() {
        return Err(DecodeError::UnexpectedEof);
    }
    match bytes[i] {
        0 => {
            let (len, ni) = read_u16(bytes, i + 1)?;
            let end = ni + len as usize;
            if end > bytes.len() {
                return Err(DecodeError::UnexpectedEof);
            }
            let s = String::from_utf8_lossy(&bytes[ni..end]).into_owned();
            Ok((DecodedValue::Str(s), end))
        }
        1 => {
            let (v, ni) = read_i64(bytes, i + 1)?;
            Ok((DecodedValue::Int(v), ni))
        }
        2 => {
            let (v, ni) = read_f64(bytes, i + 1)?;
            Ok((DecodedValue::Float(v), ni))
        }
        3 => {
            if i + 2 > bytes.len() {
                return Err(DecodeError::UnexpectedEof);
            }
            Ok((DecodedValue::Bool(bytes[i + 1] != 0), i + 2))
        }
        t => Err(DecodeError::UnknownValueTag(t)),
    }
}

/// In-process host for tests — applies decoded ops to an in-memory view tree.
#[derive(Clone, Debug, Default)]
pub struct InProcessHost {
    pub ops_log: Vec<DecodedOp>,
    pub views: std::collections::HashMap<u32, ViewState>,
    pub root_children: Vec<u32>,
}

#[derive(Clone, Debug)]
pub struct ViewState {
    pub kind: u16,
    pub props: std::collections::HashMap<u16, DecodedValue>,
    pub children: Vec<u32>,
}

impl HostCallbacks for InProcessHost {
    fn apply_ops(&mut self, bytes: &[u8]) {
        let decoded = decode_ops(bytes).expect("invalid command buffer");
        for op in decoded {
            self.apply_one(op.clone());
            self.ops_log.push(op);
        }
    }
}

impl InProcessHost {
    fn apply_one(&mut self, op: DecodedOp) {
        match op {
            DecodedOp::Create { handle, kind } => {
                self.views.insert(
                    handle,
                    ViewState {
                        kind,
                        props: std::collections::HashMap::new(),
                        children: Vec::new(),
                    },
                );
            }
            DecodedOp::SetProp { handle, key, value } => {
                if let Some(v) = self.views.get_mut(&handle) {
                    v.props.insert(key, value);
                }
            }
            DecodedOp::InsertChild { parent, child, index } => {
                if parent == u32::MAX {
                    let idx = index.min(self.root_children.len() as u32) as usize;
                    if idx <= self.root_children.len() {
                        self.root_children.insert(idx, child);
                    }
                } else if let Some(p) = self.views.get_mut(&parent) {
                    let idx = index.min(p.children.len() as u32) as usize;
                    p.children.insert(idx, child);
                }
            }
            DecodedOp::Remove { handle } => {
                self.views.remove(&handle);
            }
            DecodedOp::SetHandler { .. } => {}
        }
    }
}

/// Map IR prop keys to wire format (matches Swift `FlashPropKey` / Kotlin `FlashPropKey`).
pub fn prop_key_to_wire(key: &PropKey) -> u16 {
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

pub fn ir_prop_to_platform(key: flash_ir::PropKey) -> PropKey {
    match key {
        flash_ir::PropKey::Text => PropKey::Text,
        flash_ir::PropKey::Title => PropKey::Title,
        flash_ir::PropKey::Value => PropKey::Value,
        flash_ir::PropKey::Src => PropKey::Src,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::renderer::{CommandBuffer, CommandOp, PropValue, NodeKind};

    #[test]
    fn roundtrip_command_buffer() {
        let mut buf = CommandBuffer::new();
        buf.ops.push(CommandOp::Create {
            kind: NodeKind::TEXT,
            handle: 1,
        });
        buf.ops.push(CommandOp::SetProp {
            handle: 1,
            key: PropKey::Text,
            value: PropValue::Str("Hello".into()),
        });
        let bytes = buf.encode();
        let ops = decode_ops(&bytes).unwrap();
        assert_eq!(ops.len(), 2);
        assert!(matches!(
            &ops[1],
            DecodedOp::SetProp { value: DecodedValue::Str(s), .. } if s == "Hello"
        ));
    }

    #[test]
    fn in_process_host_creates_views() {
        let mut host = InProcessHost::default();
        let mut buf = CommandBuffer::new();
        buf.ops.push(CommandOp::Create {
            kind: NodeKind::COLUMN,
            handle: 0,
        });
        buf.ops.push(CommandOp::Create {
            kind: NodeKind::TEXT,
            handle: 1,
        });
        buf.ops.push(CommandOp::InsertChild {
            parent: 0,
            child: 1,
            index: 0,
        });
        host.apply_ops(&buf.encode());
        assert_eq!(host.views.len(), 2);
        assert_eq!(host.views.get(&0).unwrap().children, vec![1]);
    }
}
