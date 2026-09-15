//! Material Design widgets.

use super::{PrimaryProp, WidgetCategory, WidgetDef};
use super::widget;

pub const WIDGETS: &[WidgetDef] = &[
    widget!(65, "FAB", WidgetCategory::Material, "Floating action button", "UIButton", "FloatingActionButton", false, true, PrimaryProp::None, false),
    widget!(66, "ElevatedButton", WidgetCategory::Material, "Material elevated button", "UIButton", "MaterialButton", false, true, PrimaryProp::Title, true),
    widget!(67, "OutlinedButton", WidgetCategory::Material, "Material outlined button", "UIButton", "MaterialButton", false, true, PrimaryProp::Title, true),
    widget!(68, "TextButton", WidgetCategory::Material, "Material text button", "UIButton", "MaterialButton", false, true, PrimaryProp::Title, true),
    widget!(69, "BottomSheet", WidgetCategory::Material, "Material bottom sheet", "UIViewController", "BottomSheetDialog", true, false, PrimaryProp::None, false),
    widget!(70, "DropdownMenu", WidgetCategory::Material, "Overflow dropdown menu", "UIMenu", "PopupMenu", true, false, PrimaryProp::None, false),
    widget!(71, "ListItem", WidgetCategory::Material, "Material list item row", "UITableViewCell", "MaterialListItem", true, true, PrimaryProp::Title, true),
];
