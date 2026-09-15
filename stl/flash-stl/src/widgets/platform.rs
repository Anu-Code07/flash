//! Platform extension widgets — Rust FFI bridges to native SDKs.

use super::{PrimaryProp, WidgetCategory, WidgetDef};
use super::widget;

pub const WIDGETS: &[WidgetDef] = &[
    widget!(83, "WebView", WidgetCategory::Platform, "Embedded web content", "WKWebView", "WebView", false, false, PrimaryProp::Src, true),
    widget!(84, "MapView", WidgetCategory::Platform, "Native map view", "MKMapView", "MapView", false, false, PrimaryProp::None, false),
    widget!(85, "CameraPreview", WidgetCategory::Platform, "Live camera preview", "AVCaptureSession", "CameraX", false, false, PrimaryProp::None, false),
    widget!(86, "VideoPlayer", WidgetCategory::Platform, "Inline video playback", "AVPlayerView", "ExoPlayer", false, false, PrimaryProp::Src, true),
    widget!(87, "LottieAnimation", WidgetCategory::Platform, "Lottie vector animation", "LottieAnimationView", "LottieAnimationView", false, false, PrimaryProp::Src, true),
    widget!(88, "QRCode", WidgetCategory::Platform, "QR code display", "UIImageView", "ImageView", false, false, PrimaryProp::Text, true),
    widget!(89, "BarcodeScanner", WidgetCategory::Platform, "Camera barcode scanner", "AVCaptureSession", "CameraX", false, true, PrimaryProp::None, false),
    widget!(90, "ImagePicker", WidgetCategory::Platform, "Photo picker trigger", "UIImagePickerController", "ActivityResult", false, true, PrimaryProp::None, false),
    widget!(91, "ShareSheet", WidgetCategory::Platform, "Native share sheet", "UIActivityViewController", "Intent", false, true, PrimaryProp::Text, true),
];
