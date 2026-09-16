//! Flash minimal runtime — state slots, reactive flush, event dispatch.
//! Mobile-first: one FFI crossing per frame via command buffer.

pub mod state;
pub mod reactive;
pub mod renderer;
pub mod live_renderer;
pub mod hot_reload;
pub mod native;

pub use state::SlotStore;
pub use reactive::ReactiveEngine;
pub use flash_ir::PropKey;
pub use renderer::{MockRenderer, RenderCall, PropValue};
pub use live_renderer::{LiveRenderer, PatchOp};
pub use hot_reload::{DevSession, HotReloadKind, HotReloadResult};
pub use native::{NativeSession, flush_screen, mount_screen};
