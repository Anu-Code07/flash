//! Flash platform layer — mobile-first (iOS, Android) with web support.
//!
//! Architecture:
//!
//! ```text
//! UI IR -> Renderer trait -> Platform adapter
//!   - iOS:     C vtable -> Swift/UIKit
//!   - Android: JNI       -> Kotlin/View
//!   - Web:     WASM      -> DOM shim
//! ```

pub mod target;
pub mod renderer;
pub mod animation;
pub mod layout_engine;
pub mod host;
pub mod native_renderer;
pub mod ffi;

pub use target::PlatformTarget;
pub use renderer::{PlatformRenderer, CommandBuffer, CommandOp, PropKey, PropValue, NodeKind};
pub use animation::{PlatformAnimator, AnimationHandle};
pub use layout_engine::LayoutEngine;
pub use host::{HostCallbacks, InProcessHost, ViewState, decode_ops, DecodedOp, DecodedValue};
pub use native_renderer::{NativeRenderer, set_node_prop};
