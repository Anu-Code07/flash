//! Display widgets.

use super::{PrimaryProp, WidgetCategory, WidgetDef};
use super::widget;

pub const WIDGETS: &[WidgetDef] = &[
    widget!(20, "Icon", WidgetCategory::Display, "SF Symbol / Material icon", "UIImageView", "ImageView", false, false, PrimaryProp::Text, true),
    widget!(21, "Avatar", WidgetCategory::Display, "Circular profile image", "UIImageView", "ShapeableImageView", false, false, PrimaryProp::Src, true),
    widget!(22, "Badge", WidgetCategory::Display, "Notification count badge", "UIView", "TextView", true, false, PrimaryProp::Text, false),
    widget!(23, "Divider", WidgetCategory::Display, "Horizontal or vertical rule", "UIView", "View", false, false, PrimaryProp::None, false),
    widget!(24, "Card", WidgetCategory::Display, "Elevated surface container", "UIView", "MaterialCardView", true, false, PrimaryProp::None, false),
    widget!(25, "RichText", WidgetCategory::Display, "Text with inline spans", "UILabel", "SpannedString", false, false, PrimaryProp::Text, true),
    widget!(26, "ProgressBar", WidgetCategory::Display, "Linear progress indicator", "UIProgressView", "ProgressBar", false, false, PrimaryProp::Value, false),
    widget!(27, "Placeholder", WidgetCategory::Display, "Skeleton loading placeholder", "UIView", "View", false, false, PrimaryProp::None, false),
    widget!(28, "Shimmer", WidgetCategory::Display, "Animated skeleton shimmer", "UIView", "ShimmerFrameLayout", true, false, PrimaryProp::None, false),
    widget!(29, "Chip", WidgetCategory::Material, "Compact tag or filter chip", "UIView", "Chip", false, true, PrimaryProp::Text, true),
    widget!(30, "Tooltip", WidgetCategory::Feedback, "Hover/long-press hint", "UIView", "PopupWindow", true, false, PrimaryProp::Text, true),
];
