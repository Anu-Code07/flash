//! Flash Standard Library (STL)
//!
//! C++ STL-inspired standard types, containers, and built-in UI primitives
//! available to every `.ui` file without imports.

pub mod types;
pub mod builtins;
pub mod registry;

pub use types::*;
pub use builtins::*;
pub use registry::StlRegistry;
