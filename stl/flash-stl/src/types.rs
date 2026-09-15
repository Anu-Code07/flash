//! Standard types — analogous to C++ fundamental types and STL containers.

use flash_span::Symbol;
use std::fmt;

/// Unique identifier for a type in the STL.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

/// A fully-resolved type in the Flash type system.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeKind {
    // Fundamental types (like C++ arithmetic / void)
    Unit,
    Int,
    Float,
    Bool,
    String,

    // STL containers (like std::vector, std::map, std::optional)
    List(Box<TypeKind>),
    Map { key: Box<TypeKind>, value: Box<TypeKind> },
    Set(Box<TypeKind>),
    Option(Box<TypeKind>),

    // Function type (handlers only)
    Fn { params: Vec<TypeKind>, ret: Box<TypeKind> },

    // Imported from Rust via .uiapi
    Struct(Symbol),
    Enum(Symbol),

    // Component signature
    Component(Vec<ParamSig>),

    // Type error sentinel
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParamSig {
    pub name: Symbol,
    pub ty: TypeKind,
    pub optional: bool,
}

impl TypeKind {
    pub fn is_display(&self) -> bool {
        matches!(
            self,
            TypeKind::Int
                | TypeKind::Float
                | TypeKind::Bool
                | TypeKind::String
                | TypeKind::Option(_)
        )
    }

    pub fn name_str(&self) -> &'static str {
        match self {
            TypeKind::Unit => "Unit",
            TypeKind::Int => "Int",
            TypeKind::Float => "Float",
            TypeKind::Bool => "Bool",
            TypeKind::String => "String",
            TypeKind::List(_) => "List",
            TypeKind::Map { .. } => "Map",
            TypeKind::Set(_) => "Set",
            TypeKind::Option(_) => "Option",
            TypeKind::Fn { .. } => "Fn",
            TypeKind::Struct(_) => "Struct",
            TypeKind::Enum(_) => "Enum",
            TypeKind::Component(_) => "Component",
            TypeKind::Error => "Error",
        }
    }

    /// Parse a type name from `.ui` source syntax, e.g. `List<Flight>`.
    pub fn from_name(name: &str, generics: &[TypeKind]) -> Option<TypeKind> {
        match name {
            "Unit" => Some(TypeKind::Unit),
            "Int" => Some(TypeKind::Int),
            "Float" => Some(TypeKind::Float),
            "Bool" => Some(TypeKind::Bool),
            "String" => Some(TypeKind::String),
            "List" => generics.first().map(|t| TypeKind::List(Box::new(t.clone()))),
            "Set" => generics.first().map(|t| TypeKind::Set(Box::new(t.clone()))),
            "Option" => generics.first().map(|t| TypeKind::Option(Box::new(t.clone()))),
            "Map" => {
                if generics.len() >= 2 {
                    Some(TypeKind::Map {
                        key: Box::new(generics[0].clone()),
                        value: Box::new(generics[1].clone()),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

impl fmt::Display for TypeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeKind::Unit => write!(f, "Unit"),
            TypeKind::Int => write!(f, "Int"),
            TypeKind::Float => write!(f, "Float"),
            TypeKind::Bool => write!(f, "Bool"),
            TypeKind::String => write!(f, "String"),
            TypeKind::List(inner) => write!(f, "List<{}>", inner),
            TypeKind::Map { key, value } => write!(f, "Map<{}, {}>", key, value),
            TypeKind::Set(inner) => write!(f, "Set<{}>", inner),
            TypeKind::Option(inner) => write!(f, "{}?", inner),
            TypeKind::Fn { params, ret } => {
                write!(f, "Fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ") -> {}", ret.as_ref())
            }
            TypeKind::Struct(s) => write!(f, "Struct({})", s.0),
            TypeKind::Enum(s) => write!(f, "Enum({})", s.0),
            TypeKind::Component(_) => write!(f, "Component"),
            TypeKind::Error => write!(f, "<error>"),
        }
    }
}

/// Well-known primitive type IDs (stable across compilations).
pub mod primitive_ids {
    use super::TypeId;
    pub const UNIT: TypeId = TypeId(0);
    pub const INT: TypeId = TypeId(1);
    pub const FLOAT: TypeId = TypeId(2);
    pub const BOOL: TypeId = TypeId(3);
    pub const STRING: TypeId = TypeId(4);
}
