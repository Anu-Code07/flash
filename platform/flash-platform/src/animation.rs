//! Platform animation adapters — iOS Core Animation, Android Animator, Web WAAPI.

use flash_ir::animation::{AnimatableProp, AnimationSpec, Easing};

/// Platform-specific animation handle returned after one-shot setup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationHandle(pub u64);

/// Trait implemented by iOS/Android/Web adapters.
/// Called once per state change, never per frame.
pub trait PlatformAnimator {
    fn start(&mut self, spec: &AnimationSpec) -> AnimationHandle;
    fn cancel(&mut self, handle: AnimationHandle);
}

/// Maps IR easing to platform-native curve identifiers.
pub fn easing_to_platform(easing: Easing, target: super::target::PlatformTarget) -> &'static str {
    match (easing, target) {
        (Easing::Linear, _) => "linear",
        (Easing::EaseIn, super::target::PlatformTarget::Ios) => "easeIn",
        (Easing::EaseOut, super::target::PlatformTarget::Ios) => "easeOut",
        (Easing::EaseInOut, _) => "easeInOut",
        (Easing::Spring { .. }, super::target::PlatformTarget::Ios) => "spring",
        (Easing::Spring { .. }, super::target::PlatformTarget::Android) => "spring",
        (Easing::Spring { .. }, super::target::PlatformTarget::Web) => "spring",
        (Easing::EaseIn, _) => "accelerate",
        (Easing::EaseOut, _) => "decelerate",
    }
}

/// iOS: CABasicAnimation / UIViewPropertyAnimator
/// Android: ObjectAnimator on View.TRANSLATION_X etc.
/// Web: element.animate() via WAAPI
pub fn animatable_to_native_key(prop: AnimatableProp, target: super::target::PlatformTarget) -> &'static str {
    match (prop, target) {
        (AnimatableProp::Opacity, _) => "opacity",
        (AnimatableProp::TranslateX, super::target::PlatformTarget::Ios) => "transform.translation.x",
        (AnimatableProp::TranslateX, super::target::PlatformTarget::Android) => "translationX",
        (AnimatableProp::TranslateX, super::target::PlatformTarget::Web) => "translateX",
        (AnimatableProp::TranslateY, super::target::PlatformTarget::Ios) => "transform.translation.y",
        (AnimatableProp::TranslateY, super::target::PlatformTarget::Android) => "translationY",
        (AnimatableProp::TranslateY, super::target::PlatformTarget::Web) => "translateY",
        (AnimatableProp::Scale, super::target::PlatformTarget::Ios) => "transform.scale",
        (AnimatableProp::Scale, super::target::PlatformTarget::Android) => "scaleX",
        (AnimatableProp::Scale, super::target::PlatformTarget::Web) => "scale",
        (AnimatableProp::Rotate, super::target::PlatformTarget::Ios) => "transform.rotation.z",
        (AnimatableProp::Rotate, super::target::PlatformTarget::Android) => "rotation",
        (AnimatableProp::Rotate, super::target::PlatformTarget::Web) => "rotate",
    }
}
