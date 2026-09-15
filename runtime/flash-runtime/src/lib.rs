//! Flash minimal runtime — state slots, reactive flush, event dispatch.
//! Mobile-first: one FFI crossing per frame via command buffer.

pub mod state;
pub mod reactive;
pub mod renderer;

pub use state::SlotStore;
pub use reactive::ReactiveEngine;
pub use flash_ir::PropKey;
pub use renderer::{MockRenderer, RenderCall, PropValue};
