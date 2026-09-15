//! Built-in UI primitives and STL algorithms available in every `.ui` file.

use flash_span::Symbol;
use crate::types::{ParamSig, TypeKind};

/// Kind of built-in UI node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BuiltinNode {
    Text,
    Button,
    Column,
    Row,
    Stack,
    Image,
    TextField,
    ScrollView,
    List,
    Loading,
}

impl BuiltinNode {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Text" => Some(BuiltinNode::Text),
            "Button" => Some(BuiltinNode::Button),
            "Column" => Some(BuiltinNode::Column),
            "Row" => Some(BuiltinNode::Row),
            "Stack" => Some(BuiltinNode::Stack),
            "Image" => Some(BuiltinNode::Image),
            "TextField" => Some(BuiltinNode::TextField),
            "ScrollView" => Some(BuiltinNode::ScrollView),
            "List" => Some(BuiltinNode::List),
            "Loading" => Some(BuiltinNode::Loading),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            BuiltinNode::Text => "Text",
            BuiltinNode::Button => "Button",
            BuiltinNode::Column => "Column",
            BuiltinNode::Row => "Row",
            BuiltinNode::Stack => "Stack",
            BuiltinNode::Image => "Image",
            BuiltinNode::TextField => "TextField",
            BuiltinNode::ScrollView => "ScrollView",
            BuiltinNode::List => "List",
            BuiltinNode::Loading => "Loading",
        }
    }

    /// Whether the trailing block is an event handler (leaf) vs children (container).
    pub fn block_is_handler(&self) -> bool {
        matches!(self, BuiltinNode::Button | BuiltinNode::TextField)
    }

    pub fn is_container(&self) -> bool {
        matches!(
            self,
            BuiltinNode::Column
                | BuiltinNode::Row
                | BuiltinNode::Stack
                | BuiltinNode::ScrollView
                | BuiltinNode::List
        )
    }
}

/// Signature of a built-in node's parameters.
#[derive(Clone, Debug)]
pub struct BuiltinSig {
    pub node: BuiltinNode,
    pub params: Vec<ParamSig>,
    pub accepts_children: bool,
    pub accepts_handler: bool,
}

/// Built-in modifier (styling).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BuiltinModifier {
    Font,
    Color,
    Padding,
    Background,
    OnTap,
    OnSwipe,
}

impl BuiltinModifier {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "font" => Some(BuiltinModifier::Font),
            "color" => Some(BuiltinModifier::Color),
            "padding" => Some(BuiltinModifier::Padding),
            "background" => Some(BuiltinModifier::Background),
            "onTap" => Some(BuiltinModifier::OnTap),
            "onSwipe" => Some(BuiltinModifier::OnSwipe),
            _ => None,
        }
    }
}

/// STL-style algorithms exposed to `.ui` handlers via Tier-1 imports.
/// Listed here for documentation; actual implementations live in Rust.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StlAlgorithm {
    /// `len(collection)` — like `std::size`
    Len,
    /// `is_empty(collection)`
    IsEmpty,
    /// `contains(collection, item)`
    Contains,
    /// `clamp(value, min, max)`
    Clamp,
}

impl StlAlgorithm {
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "len" => Some(StlAlgorithm::Len),
            "is_empty" => Some(StlAlgorithm::IsEmpty),
            "contains" => Some(StlAlgorithm::Contains),
            "clamp" => Some(StlAlgorithm::Clamp),
            _ => None,
        }
    }
}

/// Get signatures for all built-in UI nodes.
pub fn builtin_signatures() -> Vec<BuiltinSig> {
    use TypeKind::*;
    vec![
        BuiltinSig {
            node: BuiltinNode::Text,
            params: vec![ParamSig {
                name: Symbol(0), // positional
                ty: String,
                optional: false,
            }],
            accepts_children: false,
            accepts_handler: false,
        },
        BuiltinSig {
            node: BuiltinNode::Button,
            params: vec![ParamSig {
                name: Symbol(0),
                ty: String,
                optional: false,
            }],
            accepts_children: false,
            accepts_handler: true,
        },
        BuiltinSig {
            node: BuiltinNode::Column,
            params: vec![],
            accepts_children: true,
            accepts_handler: false,
        },
        BuiltinSig {
            node: BuiltinNode::Row,
            params: vec![],
            accepts_children: true,
            accepts_handler: false,
        },
        BuiltinSig {
            node: BuiltinNode::Stack,
            params: vec![],
            accepts_children: true,
            accepts_handler: false,
        },
        BuiltinSig {
            node: BuiltinNode::Image,
            params: vec![ParamSig {
                name: Symbol(0),
                ty: String,
                optional: false,
            }],
            accepts_children: false,
            accepts_handler: false,
        },
        BuiltinSig {
            node: BuiltinNode::TextField,
            params: vec![ParamSig {
                name: Symbol(0),
                ty: String,
                optional: true,
            }],
            accepts_children: false,
            accepts_handler: true,
        },
        BuiltinSig {
            node: BuiltinNode::ScrollView,
            params: vec![],
            accepts_children: true,
            accepts_handler: false,
        },
        BuiltinSig {
            node: BuiltinNode::List,
            params: vec![ParamSig {
                name: Symbol(0),
                ty: Error, // generic — checked at call site
                optional: false,
            }],
            accepts_children: true,
            accepts_handler: false,
        },
        BuiltinSig {
            node: BuiltinNode::Loading,
            params: vec![],
            accepts_children: false,
            accepts_handler: false,
        },
    ]
}
