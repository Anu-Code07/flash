//! Modifier extensions — styling chains for mobile UI.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModifierExt {
    // Layout
    Padding,
    PaddingHorizontal,
    PaddingVertical,
    Margin,
    Width,
    Height,
    FillMaxWidth,
    FillMaxHeight,
    AspectRatio,
    // Typography
    Font,
    FontSize,
    FontWeight,
    LineHeight,
    LetterSpacing,
    TextAlign,
    // Color & surface
    Color,
    Background,
    Border,
    BorderRadius,
    Shadow,
    Opacity,
    // Flex
    Flex,
    Align,
    Justify,
    Gap,
    // Mobile
    SafeArea,
    StatusBarPadding,
    KeyboardAvoiding,
    // Interaction
    OnTap,
    OnLongPress,
    OnSwipe,
    Disabled,
    // Animation (compositor-only)
    WithAnimation,
    AnimateOpacity,
    AnimateOffset,
    AnimateScale,
}

impl ModifierExt {
    pub fn name(&self) -> &'static str {
        match self {
            ModifierExt::Padding => "padding",
            ModifierExt::PaddingHorizontal => "padding_horizontal",
            ModifierExt::PaddingVertical => "padding_vertical",
            ModifierExt::Margin => "margin",
            ModifierExt::Width => "width",
            ModifierExt::Height => "height",
            ModifierExt::FillMaxWidth => "fill_max_width",
            ModifierExt::FillMaxHeight => "fill_max_height",
            ModifierExt::AspectRatio => "aspect_ratio",
            ModifierExt::Font => "font",
            ModifierExt::FontSize => "font_size",
            ModifierExt::FontWeight => "font_weight",
            ModifierExt::LineHeight => "line_height",
            ModifierExt::LetterSpacing => "letter_spacing",
            ModifierExt::TextAlign => "text_align",
            ModifierExt::Color => "color",
            ModifierExt::Background => "background",
            ModifierExt::Border => "border",
            ModifierExt::BorderRadius => "border_radius",
            ModifierExt::Shadow => "shadow",
            ModifierExt::Opacity => "opacity",
            ModifierExt::Flex => "flex",
            ModifierExt::Align => "align",
            ModifierExt::Justify => "justify",
            ModifierExt::Gap => "gap",
            ModifierExt::SafeArea => "safe_area",
            ModifierExt::StatusBarPadding => "status_bar_padding",
            ModifierExt::KeyboardAvoiding => "keyboard_avoiding",
            ModifierExt::OnTap => "on_tap",
            ModifierExt::OnLongPress => "on_long_press",
            ModifierExt::OnSwipe => "on_swipe",
            ModifierExt::Disabled => "disabled",
            ModifierExt::WithAnimation => "with_animation",
            ModifierExt::AnimateOpacity => "animate_opacity",
            ModifierExt::AnimateOffset => "animate_offset",
            ModifierExt::AnimateScale => "animate_scale",
        }
    }

    pub fn is_animatable(&self) -> bool {
        matches!(
            self,
            ModifierExt::Opacity | ModifierExt::WithAnimation
                | ModifierExt::AnimateOpacity | ModifierExt::AnimateOffset
                | ModifierExt::AnimateScale
        )
    }

    pub fn all() -> &'static [ModifierExt] {
        &[
            ModifierExt::Padding, ModifierExt::PaddingHorizontal, ModifierExt::PaddingVertical,
            ModifierExt::Margin, ModifierExt::Width, ModifierExt::Height,
            ModifierExt::FillMaxWidth, ModifierExt::FillMaxHeight, ModifierExt::AspectRatio,
            ModifierExt::Font, ModifierExt::FontSize, ModifierExt::FontWeight,
            ModifierExt::LineHeight, ModifierExt::LetterSpacing, ModifierExt::TextAlign,
            ModifierExt::Color, ModifierExt::Background, ModifierExt::Border,
            ModifierExt::BorderRadius, ModifierExt::Shadow, ModifierExt::Opacity,
            ModifierExt::Flex, ModifierExt::Align, ModifierExt::Justify, ModifierExt::Gap,
            ModifierExt::SafeArea, ModifierExt::StatusBarPadding, ModifierExt::KeyboardAvoiding,
            ModifierExt::OnTap, ModifierExt::OnLongPress, ModifierExt::OnSwipe, ModifierExt::Disabled,
            ModifierExt::WithAnimation, ModifierExt::AnimateOpacity,
            ModifierExt::AnimateOffset, ModifierExt::AnimateScale,
        ]
    }
}
