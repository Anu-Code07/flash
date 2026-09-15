//! Flash Standard Library (STL)
//!
//! C++ STL-inspired standard types, containers, and built-in UI primitives
//! available to every `.ui` file without imports.

pub mod types;
pub mod builtins;
pub mod registry;
pub mod collections;
pub mod string;
pub mod math;
pub mod mobile;
pub mod modifiers;
pub mod extensions;
pub mod logger;

pub use types::*;
pub use builtins::*;
pub use registry::StlRegistry;
pub use collections::CollectionFn;
pub use string::StringFn;
pub use math::MathFn;
pub use mobile::MobileFn;
pub use modifiers::ModifierExt;
pub use extensions::{AnimationCurve, DesignToken, StlCatalog};
