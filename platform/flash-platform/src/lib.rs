//! Flash platform layer — mobile-first (iOS, Android) with web support.
//!
//! Architecture:
//! ```
//! UI IR → Renderer trait → Platform adapter
//!   ├── iOS:     C vtable → Swift/UIKit
//!   ├── Android: JNI       → Kotlin/View
//!   └── Web:     WASM      → DOM shim
//! ```

pub mod target;
pub mod renderer;
pub mod animation;
pub mod layout_engine;

pub use target::PlatformTarget;
pub use renderer::{PlatformRenderer, CommandBuffer, PropKey, PropValue, NodeKind};
pub use animation::{PlatformAnimator, AnimationHandle};
pub use layout_engine::LayoutEngine;
