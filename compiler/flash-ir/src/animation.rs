//! Animation IR — platform-native animators, zero per-frame crossings.
//!
//! Animations are NOT update ops. They compile to one-shot native animation
//! descriptions (Core Animation, Android Animator, WAAPI).

/// Properties safe to animate without triggering layout (compositor-only).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimatableProp {
    Opacity,
    TranslateX,
    TranslateY,
    Scale,
    Rotate,
}

impl AnimatableProp {
    pub fn is_animatable(name: &str) -> bool {
        matches!(
            name,
            "opacity" | "translateX" | "translateY" | "scale" | "rotate" | "offset"
        )
    }

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "opacity" => Some(AnimatableProp::Opacity),
            "translateX" => Some(AnimatableProp::TranslateX),
            "translateY" => Some(AnimatableProp::TranslateY),
            "scale" => Some(AnimatableProp::Scale),
            "rotate" => Some(AnimatableProp::Rotate),
            _ => None,
        }
    }
}

/// Layout properties must never be driven per-frame through reactive flush.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayoutProp {
    Width,
    Height,
    Padding,
    Margin,
    FontSize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Spring { damping: f32, stiffness: f32 },
}

#[derive(Clone, Debug)]
pub struct AnimationSpec {
    pub id: u32,
    pub nodes: Vec<super::NodeId>,
    pub property: AnimatableProp,
    /// Source slot — compiler resolves variant → from/to values at compile time.
    pub source_slot: super::SlotId,
    pub from: f64,
    pub to: f64,
    pub duration_ms: u32,
    pub easing: Easing,
}

/// One animation setup per state change — NOT per frame.
#[derive(Clone, Debug)]
pub enum AnimationCommand {
    Start(AnimationSpec),
    Cancel { id: u32 },
}
