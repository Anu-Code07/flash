//! C ABI exports for iOS static lib linking.
//!
//! Swift registers a vtable at startup; Rust calls into UIKit on each `commit()`.

use std::sync::Mutex;

use crate::host::HostCallbacks;

type CreateFn = extern "C" fn(handle: u32, kind: u16);
type SetPropFn = extern "C" fn(handle: u32, key: u16, ptr: *const u8, len: u32);
type InsertChildFn = extern "C" fn(parent: u32, child: u32, index: u32);
type RemoveFn = extern "C" fn(handle: u32);
type SetHandlerFn = extern "C" fn(handle: u32, handler_id: u32);
type SetFrameFn = extern "C" fn(handle: u32, x: f32, y: f32, width: f32, height: f32);
type CommitFn = extern "C" fn();

/// Function pointers registered by the Swift `FlashHost` at app launch.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FlashHostVTable {
    pub create: CreateFn,
    pub set_prop: SetPropFn,
    pub insert_child: InsertChildFn,
    pub remove: RemoveFn,
    pub set_handler: SetHandlerFn,
    pub set_frame: SetFrameFn,
    pub commit: CommitFn,
}

static VTABLE: Mutex<Option<FlashHostVTable>> = Mutex::new(None);

/// Called from Swift: `flash_host_register(&vtable)`
#[no_mangle]
pub extern "C" fn flash_host_register(vtable: FlashHostVTable) {
    *VTABLE.lock().unwrap() = Some(vtable);
}

/// Called from Swift: apply a full encoded command buffer (batch path).
#[no_mangle]
pub extern "C" fn flash_host_apply_ops(ptr: *const u8, len: u32) {
    if ptr.is_null() || len == 0 {
        return;
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    if let Ok(ops) = crate::host::decode_ops(bytes) {
        let vt = *VTABLE.lock().unwrap();
        if let Some(vt) = vt {
            for op in ops {
                apply_decoded(&vt, op);
            }
            (vt.commit)();
        }
    }
}

fn apply_decoded(vt: &FlashHostVTable, op: crate::host::DecodedOp) {
    match op {
        crate::host::DecodedOp::Create { handle, kind } => (vt.create)(handle, kind),
        crate::host::DecodedOp::SetProp { handle, key, value } => {
            let bytes = encode_value_for_c(&value);
            (vt.set_prop)(handle, key, bytes.as_ptr(), bytes.len() as u32);
        }
        crate::host::DecodedOp::InsertChild { parent, child, index } => {
            (vt.insert_child)(parent, child, index);
        }
        crate::host::DecodedOp::Remove { handle } => (vt.remove)(handle),
        crate::host::DecodedOp::SetHandler { handle, handler_id } => {
            (vt.set_handler)(handle, handler_id);
        }
        crate::host::DecodedOp::SetFrame { handle, x, y, width, height } => {
            (vt.set_frame)(handle, x, y, width, height);
        }
    }
}

fn encode_value_for_c(value: &crate::host::DecodedValue) -> Vec<u8> {
    match value {
        crate::host::DecodedValue::Str(s) => s.as_bytes().to_vec(),
        crate::host::DecodedValue::Int(v) => v.to_le_bytes().to_vec(),
        crate::host::DecodedValue::Float(v) => v.to_le_bytes().to_vec(),
        crate::host::DecodedValue::Bool(b) => vec![if *b { 1 } else { 0 }],
    }
}

/// Rust-side host that forwards encoded buffers to the registered C vtable.
pub struct CHostBridge;

impl HostCallbacks for CHostBridge {
    fn apply_ops(&mut self, ops: &[u8]) {
        flash_host_apply_ops(ops.as_ptr(), ops.len() as u32);
    }
}
