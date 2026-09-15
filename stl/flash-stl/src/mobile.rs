//! Mobile-specific STL — safe areas, platform detection, haptics, lifecycle.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MobileFn {
    SafeAreaTop,
    SafeAreaBottom,
    SafeAreaLeft,
    SafeAreaRight,
    ScreenWidth,
    ScreenHeight,
    IsIos,
    IsAndroid,
    IsWeb,
    IsTablet,
    IsPhone,
    IsDarkMode,
    IsLandscape,
    HapticLight,
    HapticMedium,
    HapticHeavy,
    KeyboardHeight,
    StatusBarHeight,
    Navigate,
    NavigateBack,
    NavigateReplace,
    Share,
    OpenUrl,
}

impl MobileFn {
    pub fn name(&self) -> &'static str {
        match self {
            MobileFn::SafeAreaTop => "safe_area_top",
            MobileFn::SafeAreaBottom => "safe_area_bottom",
            MobileFn::SafeAreaLeft => "safe_area_left",
            MobileFn::SafeAreaRight => "safe_area_right",
            MobileFn::ScreenWidth => "screen_width",
            MobileFn::ScreenHeight => "screen_height",
            MobileFn::IsIos => "is_ios",
            MobileFn::IsAndroid => "is_android",
            MobileFn::IsWeb => "is_web",
            MobileFn::IsTablet => "is_tablet",
            MobileFn::IsPhone => "is_phone",
            MobileFn::IsDarkMode => "is_dark_mode",
            MobileFn::IsLandscape => "is_landscape",
            MobileFn::HapticLight => "haptic_light",
            MobileFn::HapticMedium => "haptic_medium",
            MobileFn::HapticHeavy => "haptic_heavy",
            MobileFn::KeyboardHeight => "keyboard_height",
            MobileFn::StatusBarHeight => "status_bar_height",
            MobileFn::Navigate => "navigate",
            MobileFn::NavigateBack => "navigate_back",
            MobileFn::NavigateReplace => "navigate_replace",
            MobileFn::Share => "share",
            MobileFn::OpenUrl => "open_url",
        }
    }

    pub fn all() -> &'static [MobileFn] {
        &[
            MobileFn::SafeAreaTop, MobileFn::SafeAreaBottom,
            MobileFn::SafeAreaLeft, MobileFn::SafeAreaRight,
            MobileFn::ScreenWidth, MobileFn::ScreenHeight,
            MobileFn::IsIos, MobileFn::IsAndroid, MobileFn::IsWeb,
            MobileFn::IsTablet, MobileFn::IsPhone,
            MobileFn::IsDarkMode, MobileFn::IsLandscape,
            MobileFn::HapticLight, MobileFn::HapticMedium, MobileFn::HapticHeavy,
            MobileFn::KeyboardHeight, MobileFn::StatusBarHeight,
            MobileFn::Navigate, MobileFn::NavigateBack, MobileFn::NavigateReplace,
            MobileFn::Share, MobileFn::OpenUrl,
        ]
    }
}
