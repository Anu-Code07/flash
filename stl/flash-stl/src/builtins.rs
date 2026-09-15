//! Built-in UI primitives — delegates to the widget catalog in `widgets.rs`.

use crate::types::ParamSig;
use crate::widgets::{
    block_is_handler, is_container, widget_by_id, widget_by_name, widget_signatures, PrimaryProp,
    WidgetId, WidgetSig,
};

/// Kind of built-in UI node (stable ID from widget catalog).
pub type BuiltinNode = WidgetId;

impl BuiltinNode {
    pub fn from_name(name: &str) -> Option<Self> {
        widget_by_name(name).map(|w| w.id)
    }

    pub fn name(&self) -> &'static str {
        widget_by_id(*self).map(|w| w.name).unwrap_or("Unknown")
    }

    pub fn block_is_handler(&self) -> bool {
        block_is_handler(self.name())
    }

    pub fn is_container(&self) -> bool {
        is_container(self.name())
    }

    pub fn primary_prop(&self) -> PrimaryProp {
        widget_by_id(*self)
            .map(|w| w.primary_prop)
            .unwrap_or(PrimaryProp::None)
    }
}

/// Signature of a built-in node's parameters.
#[derive(Clone, Debug)]
pub struct BuiltinSig {
    pub node: BuiltinNode,
    pub params: Vec<ParamSig>,
    pub accepts_children: bool,
    pub accepts_handler: bool,
    pub primary_prop: PrimaryProp,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StlAlgorithm {
    Len,
    IsEmpty,
    Contains,
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
    widget_signatures()
        .into_iter()
        .map(|sig: WidgetSig| BuiltinSig {
            node: sig.id,
            params: sig.params,
            accepts_children: sig.accepts_children,
            accepts_handler: sig.accepts_handler,
            primary_prop: sig.primary_prop,
        })
        .collect()
}
