//! Platform target selection for mobile-first builds.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlatformTarget {
    /// iOS — UIKit via static lib + C FFI (aarch64-apple-ios)
    Ios,
    /// Android — View system via JNI + .so (aarch64-linux-android)
    Android,
    /// Web — WASM + DOM shim (wasm32-unknown-unknown)
    Web,
}

impl PlatformTarget {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ios" => Some(PlatformTarget::Ios),
            "android" => Some(PlatformTarget::Android),
            "web" => Some(PlatformTarget::Web),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            PlatformTarget::Ios => "ios",
            PlatformTarget::Android => "android",
            PlatformTarget::Web => "web",
        }
    }

    /// Rust compilation target triple.
    pub fn triple(&self) -> &'static str {
        match self {
            PlatformTarget::Ios => "aarch64-apple-ios",
            PlatformTarget::Android => "aarch64-linux-android",
            PlatformTarget::Web => "wasm32-unknown-unknown",
        }
    }

    /// Native UI backend for this target.
    pub fn native_backend(&self) -> &'static str {
        match self {
            PlatformTarget::Ios => "UIKit",
            PlatformTarget::Android => "Android View",
            PlatformTarget::Web => "DOM",
        }
    }

    /// FFI boundary mechanism.
    pub fn ffi_mechanism(&self) -> &'static str {
        match self {
            PlatformTarget::Ios => "C static lib + vtable",
            PlatformTarget::Android => "JNI + direct ByteBuffer",
            PlatformTarget::Web => "wasm-bindgen + JS shim",
        }
    }

    /// Frame driver (vsync source).
    pub fn frame_driver(&self) -> &'static str {
        match self {
            PlatformTarget::Ios => "CADisplayLink",
            PlatformTarget::Android => "Choreographer",
            PlatformTarget::Web => "requestAnimationFrame",
        }
    }

    pub fn all_mobile() -> [PlatformTarget; 2] {
        [PlatformTarget::Ios, PlatformTarget::Android]
    }

    pub fn all() -> [PlatformTarget; 3] {
        [PlatformTarget::Ios, PlatformTarget::Android, PlatformTarget::Web]
    }
}
