//! Input widgets.

use super::{PrimaryProp, WidgetCategory, WidgetDef};
use super::widget;

pub const WIDGETS: &[WidgetDef] = &[
    widget!(31, "Switch", WidgetCategory::Input, "On/off toggle", "UISwitch", "SwitchMaterial", false, true, PrimaryProp::Value, false),
    widget!(32, "Checkbox", WidgetCategory::Input, "Boolean checkbox", "UIButton", "CheckBox", false, true, PrimaryProp::Value, false),
    widget!(33, "Radio", WidgetCategory::Input, "Single selection radio", "UIButton", "RadioButton", false, true, PrimaryProp::Value, false),
    widget!(34, "RadioGroup", WidgetCategory::Input, "Group of radio buttons", "UIStackView", "RadioGroup", true, false, PrimaryProp::None, false),
    widget!(35, "Slider", WidgetCategory::Input, "Continuous value slider", "UISlider", "SeekBar", false, true, PrimaryProp::Value, false),
    widget!(36, "Picker", WidgetCategory::Input, "Dropdown / wheel picker", "UIPickerView", "Spinner", false, true, PrimaryProp::Value, false),
    widget!(37, "SegmentedControl", WidgetCategory::Input, "Mutually exclusive segments", "UISegmentedControl", "TabLayout", false, true, PrimaryProp::Value, false),
    widget!(38, "SearchBar", WidgetCategory::Input, "Search text field with icon", "UISearchBar", "SearchView", false, true, PrimaryProp::Value, false),
    widget!(39, "PasswordField", WidgetCategory::Input, "Masked text input", "UITextField", "TextInputEditText", false, true, PrimaryProp::Value, false),
    widget!(40, "IconButton", WidgetCategory::Input, "Icon-only tap target", "UIButton", "ImageButton", false, true, PrimaryProp::Text, true),
    widget!(41, "ToggleButton", WidgetCategory::Input, "Toggleable button", "UIButton", "ToggleButton", false, true, PrimaryProp::Title, true),
    widget!(42, "Stepper", WidgetCategory::Input, "Increment/decrement control", "UIStepper", "NumberPicker", false, true, PrimaryProp::Value, false),
    widget!(43, "RatingBar", WidgetCategory::Input, "Star rating input", "UIView", "RatingBar", false, true, PrimaryProp::Value, false),
    widget!(44, "DatePicker", WidgetCategory::Input, "Date selection control", "UIDatePicker", "DatePicker", false, true, PrimaryProp::Value, false),
    widget!(45, "TimePicker", WidgetCategory::Input, "Time selection control", "UIDatePicker", "TimePicker", false, true, PrimaryProp::Value, false),
];
