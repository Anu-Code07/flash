//! Feedback widgets — alerts, toasts, banners.

use super::{PrimaryProp, WidgetCategory, WidgetDef};
use super::widget;

pub const WIDGETS: &[WidgetDef] = &[
    widget!(61, "Alert", WidgetCategory::Feedback, "Modal alert dialog", "UIAlertController", "AlertDialog", true, false, PrimaryProp::Title, false),
    widget!(62, "Snackbar", WidgetCategory::Feedback, "Brief bottom message", "UIView", "Snackbar", false, false, PrimaryProp::Text, true),
    widget!(63, "Banner", WidgetCategory::Feedback, "Top informational banner", "UIView", "MaterialBanner", true, false, PrimaryProp::Text, true),
    widget!(64, "Toast", WidgetCategory::Feedback, "Transient toast message", "UIView", "Toast", false, false, PrimaryProp::Text, true),
];
