//! Cupertino (iOS HIG) widgets.

use super::{PrimaryProp, WidgetCategory, WidgetDef};
use super::widget;

pub const WIDGETS: &[WidgetDef] = &[
    widget!(72, "CupertinoButton", WidgetCategory::Cupertino, "iOS-style button", "UIButton", "MaterialButton", false, true, PrimaryProp::Title, true),
    widget!(73, "CupertinoNavigationBar", WidgetCategory::Cupertino, "iOS navigation bar", "UINavigationBar", "Toolbar", true, false, PrimaryProp::Title, false),
    widget!(74, "CupertinoTabBar", WidgetCategory::Cupertino, "iOS tab bar", "UITabBar", "BottomNavigationView", true, false, PrimaryProp::None, false),
    widget!(75, "CupertinoSwitch", WidgetCategory::Cupertino, "iOS switch", "UISwitch", "SwitchMaterial", false, true, PrimaryProp::Value, false),
    widget!(76, "CupertinoPicker", WidgetCategory::Cupertino, "iOS wheel picker", "UIPickerView", "Spinner", false, true, PrimaryProp::Value, false),
    widget!(77, "CupertinoActionSheet", WidgetCategory::Cupertino, "iOS action sheet", "UIAlertController", "BottomSheetDialog", true, false, PrimaryProp::None, false),
    widget!(78, "CupertinoAlert", WidgetCategory::Cupertino, "iOS alert dialog", "UIAlertController", "AlertDialog", true, false, PrimaryProp::Title, false),
    widget!(79, "CupertinoSearchField", WidgetCategory::Cupertino, "iOS search field", "UISearchBar", "SearchView", false, true, PrimaryProp::Value, false),
    widget!(80, "CupertinoSlider", WidgetCategory::Cupertino, "iOS slider", "UISlider", "SeekBar", false, true, PrimaryProp::Value, false),
    widget!(81, "CupertinoSegmentedControl", WidgetCategory::Cupertino, "iOS segmented control", "UISegmentedControl", "TabLayout", false, true, PrimaryProp::Value, false),
    widget!(82, "CupertinoActivityIndicator", WidgetCategory::Cupertino, "iOS spinner", "UIActivityIndicator", "ProgressBar", false, false, PrimaryProp::None, false),
];
