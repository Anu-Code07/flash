//! Global handler dispatch — native button taps → ReactiveEngine.

use std::sync::{Arc, Mutex};

use flash_ir::HandlerId;

use crate::native::NativeSession;

static ACTIVE: Mutex<Option<Arc<Mutex<NativeSession>>>> = Mutex::new(None);

/// Register session for native handler callbacks (Swift/Kotlin button taps).
pub fn register_session(session: Arc<Mutex<NativeSession>>) {
    *ACTIVE.lock().unwrap() = Some(session);
}

pub fn unregister_session() {
    *ACTIVE.lock().unwrap() = None;
}

/// Fire a handler on the active session. Returns true if a session was registered.
pub fn dispatch_handler(handler_id: u32) -> bool {
    let guard = ACTIVE.lock().unwrap();
    if let Some(session) = guard.as_ref() {
        if let Ok(mut s) = session.lock() {
            s.fire_handler(HandlerId(handler_id));
            return true;
        }
    }
    false
}

/// C ABI — Swift `FlashHost` / Android JNI call this on button tap.
#[no_mangle]
pub extern "C" fn flash_fire_handler(handler_id: u32) {
    dispatch_handler(handler_id);
}
