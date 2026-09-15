//! Navigation widgets.

use super::{PrimaryProp, WidgetCategory, WidgetDef};
use super::widget;

pub const WIDGETS: &[WidgetDef] = &[
    widget!(51, "AppBar", WidgetCategory::Navigation, "Top navigation bar", "UINavigationBar", "Toolbar", true, false, PrimaryProp::Title, false),
    widget!(52, "TabBar", WidgetCategory::Navigation, "Bottom tab navigation", "UITabBar", "BottomNavigationView", true, false, PrimaryProp::None, false),
    widget!(53, "NavBar", WidgetCategory::Navigation, "Cupertino-style nav bar", "UINavigationBar", "Toolbar", true, false, PrimaryProp::Title, false),
    widget!(54, "Modal", WidgetCategory::Navigation, "Full-screen modal overlay", "UIViewController", "DialogFragment", true, false, PrimaryProp::None, false),
    widget!(55, "Sheet", WidgetCategory::Navigation, "Bottom sheet overlay", "UIViewController", "BottomSheetDialog", true, false, PrimaryProp::None, false),
    widget!(56, "ActionSheet", WidgetCategory::Navigation, "Platform action sheet", "UIAlertController", "BottomSheetDialog", true, false, PrimaryProp::None, false),
    widget!(57, "Drawer", WidgetCategory::Navigation, "Side navigation drawer", "UIViewController", "DrawerLayout", true, false, PrimaryProp::None, false),
    widget!(58, "NavigationLink", WidgetCategory::Navigation, "Tappable navigation row", "UITableViewCell", "MaterialListItem", false, true, PrimaryProp::Title, true),
    widget!(59, "BackButton", WidgetCategory::Navigation, "Navigate back control", "UIButton", "ImageButton", false, true, PrimaryProp::None, false),
    widget!(60, "Toolbar", WidgetCategory::Navigation, "Action toolbar row", "UIToolbar", "Toolbar", true, false, PrimaryProp::None, false),
];
