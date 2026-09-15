//! Flash widget catalog — organized by category, aggregated at compile time.

mod core;
mod layout;
mod display;
mod input;
mod lists;
mod navigation;
mod feedback;
mod material;
mod cupertino;
mod platform;
mod flash;

use crate::types::{ParamSig, TypeKind};

/// Stable widget identifier (matches `flash_ir::NodeKind` discriminant).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WidgetCategory {
    Layout,
    Display,
    Input,
    Lists,
    Navigation,
    Feedback,
    Material,
    Cupertino,
    Platform,
    /// Flash-exclusive widgets — solve real problems Flutter/RN can't easily.
    Flash,
}

impl WidgetCategory {
    pub fn name(&self) -> &'static str {
        match self {
            WidgetCategory::Layout => "layout",
            WidgetCategory::Display => "display",
            WidgetCategory::Input => "input",
            WidgetCategory::Lists => "lists",
            WidgetCategory::Navigation => "navigation",
            WidgetCategory::Feedback => "feedback",
            WidgetCategory::Material => "material",
            WidgetCategory::Cupertino => "cupertino",
            WidgetCategory::Platform => "platform",
            WidgetCategory::Flash => "flash",
        }
    }
}

/// Which positional argument maps to a reactive/static prop during lowering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimaryProp {
    None,
    Text,
    Title,
    Src,
    Value,
}

#[derive(Clone, Copy, Debug)]
pub struct WidgetDef {
    pub id: WidgetId,
    pub name: &'static str,
    pub category: WidgetCategory,
    pub description: &'static str,
    pub ios: &'static str,
    pub android: &'static str,
    pub accepts_children: bool,
    pub accepts_handler: bool,
    pub primary_prop: PrimaryProp,
    pub arg_required: bool,
}

/// Macro used by category modules to declare widget entries.
macro_rules! widget {
    (
        $id:expr, $name:expr, $cat:expr, $desc:expr,
        $ios:expr, $android:expr,
        $children:expr, $handler:expr, $prop:expr, $required:expr
    ) => {
        super::WidgetDef {
            id: super::WidgetId($id),
            name: $name,
            category: $cat,
            description: $desc,
            ios: $ios,
            android: $android,
            accepts_children: $children,
            accepts_handler: $handler,
            primary_prop: $prop,
            arg_required: $required,
        }
    };
}

pub(crate) use widget;

fn category_slices() -> [&'static [WidgetDef]; 11] {
    [
        core::WIDGETS,
        layout::WIDGETS,
        display::WIDGETS,
        input::WIDGETS,
        lists::WIDGETS,
        navigation::WIDGETS,
        feedback::WIDGETS,
        material::WIDGETS,
        cupertino::WIDGETS,
        platform::WIDGETS,
        flash::WIDGETS,
    ]
}

/// Look up a widget by PascalCase name (as used in `.ui` source).
pub fn widget_by_name(name: &str) -> Option<&'static WidgetDef> {
    for slice in category_slices() {
        if let Some(w) = slice.iter().find(|w| w.name == name) {
            return Some(w);
        }
    }
    None
}

/// Look up a widget by stable ID.
pub fn widget_by_id(id: WidgetId) -> Option<&'static WidgetDef> {
    for slice in category_slices() {
        if let Some(w) = slice.iter().find(|w| w.id == id) {
            return Some(w);
        }
    }
    None
}

/// Whether a trailing `{ }` block is an event handler vs child nodes.
pub fn block_is_handler(name: &str) -> bool {
    widget_by_name(name).map(|w| w.accepts_handler).unwrap_or(false)
}

/// Whether the widget accepts nested child nodes.
pub fn is_container(name: &str) -> bool {
    widget_by_name(name)
        .map(|w| w.accepts_children)
        .unwrap_or(false)
}

/// Total number of registered widgets.
pub fn widget_count() -> usize {
    category_slices().iter().map(|s| s.len()).sum()
}

/// Export widget catalog as JSON for the documentation site.
pub fn export_widgets_json() -> String {
    use std::fmt::Write;
    let mut out = String::from("{\n  \"count\": ");
    let _ = write!(out, "{}", widget_count());
    out.push_str(",\n  \"widgets\": [\n");
    let mut first = true;
    for slice in category_slices() {
        for w in slice {
            if !first {
                out.push(',');
            }
            first = false;
            let _ = writeln!(
                out,
                "    {{\"name\": \"{}\", \"category\": \"{}\", \"description\": \"{}\", \"ios\": \"{}\", \"android\": \"{}\", \"accepts_children\": {}, \"accepts_handler\": {}, \"primary_prop\": \"{}\"}}",
                w.name,
                w.category.name(),
                w.description,
                w.ios,
                w.android,
                w.accepts_children,
                w.accepts_handler,
                primary_prop_name(w.primary_prop),
            );
        }
    }
    out.push_str("\n  ]\n}\n");
    out
}

fn primary_prop_name(prop: PrimaryProp) -> &'static str {
    match prop {
        PrimaryProp::None => "none",
        PrimaryProp::Text => "text",
        PrimaryProp::Title => "title",
        PrimaryProp::Src => "src",
        PrimaryProp::Value => "value",
    }
}

/// Build STL builtin signatures from the widget catalog.
pub fn widget_signatures() -> Vec<WidgetSig> {
    let mut sigs = Vec::with_capacity(widget_count());
    for slice in category_slices() {
        for w in slice {
            let params = match w.primary_prop {
                PrimaryProp::None => vec![],
                PrimaryProp::Text | PrimaryProp::Title | PrimaryProp::Src | PrimaryProp::Value => {
                    vec![ParamSig {
                        name: flash_span::Symbol(0),
                        ty: TypeKind::String,
                        optional: !w.arg_required,
                    }]
                }
            };
            sigs.push(WidgetSig {
                id: w.id,
                name: w.name,
                params,
                accepts_children: w.accepts_children,
                accepts_handler: w.accepts_handler,
                primary_prop: w.primary_prop,
            });
        }
    }
    sigs
}

/// Signature of a built-in widget for the STL registry.
#[derive(Clone, Debug)]
pub struct WidgetSig {
    pub id: WidgetId,
    pub name: &'static str,
    pub params: Vec<ParamSig>,
    pub accepts_children: bool,
    pub accepts_handler: bool,
    pub primary_prop: PrimaryProp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_widgets_keep_stable_ids() {
        assert_eq!(widget_by_name("Text").unwrap().id, WidgetId(0));
        assert_eq!(widget_by_name("Loading").unwrap().id, WidgetId(9));
    }

    #[test]
    fn handler_widgets() {
        assert!(block_is_handler("Button"));
        assert!(block_is_handler("Switch"));
        assert!(block_is_handler("FAB"));
        assert!(!block_is_handler("Column"));
        assert!(!block_is_handler("Text"));
    }

    #[test]
    fn catalog_has_80_plus_widgets() {
        assert!(widget_count() >= 80);
    }

    #[test]
    fn flash_exclusive_widgets_exist() {
        assert!(widget_by_name("AsyncView").is_some());
        assert!(widget_by_name("PinField").is_some());
        assert!(widget_by_name("SwipeActionRow").is_some());
        assert_eq!(
            widget_by_name("AsyncView").unwrap().category,
            WidgetCategory::Flash
        );
    }

    #[test]
    fn export_widgets_json_for_site() {
        let json = export_widgets_json();
        assert!(json.contains("\"Switch\""));
        assert!(json.contains("\"AppBar\""));
        for base in ["site/api/widgets.json", "../site/api/widgets.json", "../../site/api/widgets.json"] {
            let path = std::path::Path::new(base);
            if path.parent().map(|p| p.exists()).unwrap_or(false) {
                std::fs::write(path, &json).expect("write widgets.json");
                break;
            }
        }
    }
}
