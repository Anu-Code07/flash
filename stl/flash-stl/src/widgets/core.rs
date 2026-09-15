//! Core widgets — stable IDs 0–9 since Phase 2.

use super::{PrimaryProp, WidgetCategory, WidgetDef};
use super::widget;

pub const WIDGETS: &[WidgetDef] = &[
    widget!(0, "Text", WidgetCategory::Display, "Static or bound text", "UILabel", "TextView", false, false, PrimaryProp::Text, true),
    widget!(1, "Button", WidgetCategory::Input, "Tappable button", "UIButton", "MaterialButton", false, true, PrimaryProp::Title, true),
    widget!(2, "Column", WidgetCategory::Layout, "Vertical flex container", "UIStackView", "LinearLayout", true, false, PrimaryProp::None, false),
    widget!(3, "Row", WidgetCategory::Layout, "Horizontal flex container", "UIStackView", "LinearLayout", true, false, PrimaryProp::None, false),
    widget!(4, "Stack", WidgetCategory::Layout, "Overlapping z-ordered children", "UIView", "FrameLayout", true, false, PrimaryProp::None, false),
    widget!(5, "Image", WidgetCategory::Display, "Local or remote image", "UIImageView", "ImageView", false, false, PrimaryProp::Src, true),
    widget!(6, "TextField", WidgetCategory::Input, "Single-line text input", "UITextField", "EditText", false, true, PrimaryProp::Value, false),
    widget!(7, "ScrollView", WidgetCategory::Layout, "Scrollable single child", "UIScrollView", "ScrollView", true, false, PrimaryProp::None, false),
    widget!(8, "List", WidgetCategory::Lists, "Keyed virtualized list", "UICollectionView", "RecyclerView", true, false, PrimaryProp::None, false),
    widget!(9, "Loading", WidgetCategory::Display, "Circular loading indicator", "UIActivityIndicator", "ProgressBar", false, false, PrimaryProp::None, false),
];
