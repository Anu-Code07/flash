//! Layout widgets.

use super::{PrimaryProp, WidgetCategory, WidgetDef};
use super::widget;

pub const WIDGETS: &[WidgetDef] = &[
    widget!(10, "Wrap", WidgetCategory::Layout, "Flow layout with line breaks", "UIStackView", "FlexboxLayout", true, false, PrimaryProp::None, false),
    widget!(11, "Grid", WidgetCategory::Layout, "2D grid of children", "UICollectionView", "GridLayout", true, false, PrimaryProp::None, false),
    widget!(12, "Spacer", WidgetCategory::Layout, "Flexible empty space", "UIView", "Space", false, false, PrimaryProp::None, false),
    widget!(13, "SafeArea", WidgetCategory::Layout, "Insets for notch/home indicator", "SafeAreaLayoutGuide", "WindowInsets", true, false, PrimaryProp::None, false),
    widget!(14, "AspectRatio", WidgetCategory::Layout, "Child constrained to aspect ratio", "UIView", "ConstraintLayout", true, false, PrimaryProp::None, false),
    widget!(15, "Center", WidgetCategory::Layout, "Centers child (prefer .align modifier)", "UIView", "FrameLayout", true, false, PrimaryProp::None, false),
    widget!(16, "Expanded", WidgetCategory::Layout, "Child fills remaining flex space", "UIView", "LinearLayout", true, false, PrimaryProp::None, false),
    widget!(17, "Flexible", WidgetCategory::Layout, "Child shares flex space", "UIView", "LinearLayout", true, false, PrimaryProp::None, false),
    widget!(18, "PageView", WidgetCategory::Layout, "Horizontally paged children", "UIPageViewController", "ViewPager2", true, false, PrimaryProp::None, false),
    widget!(19, "Carousel", WidgetCategory::Layout, "Auto-advancing page carousel", "UIPageViewController", "ViewPager2", true, false, PrimaryProp::None, false),
];
