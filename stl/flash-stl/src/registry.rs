//! STL type registry — the compiler's view of all standard types.

use flash_span::{Interner, Symbol};
use crate::builtins::{builtin_signatures, BuiltinNode, BuiltinSig};
use crate::types::{TypeId, TypeKind};

/// Central registry of all STL types and built-ins.
/// Analogous to C++'s `<type_traits>` + standard headers combined.
pub struct StlRegistry {
    types: Vec<TypeKind>,
    type_names: Vec<Symbol>,
    builtins: Vec<BuiltinSig>,
    interner: Interner,
}

impl StlRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            types: Vec::new(),
            type_names: Vec::new(),
            builtins: builtin_signatures(),
            interner: Interner::new(),
        };
        reg.register_primitive("Unit", TypeKind::Unit);
        reg.register_primitive("Int", TypeKind::Int);
        reg.register_primitive("Float", TypeKind::Float);
        reg.register_primitive("Bool", TypeKind::Bool);
        reg.register_primitive("String", TypeKind::String);
        reg
    }

    fn register_primitive(&mut self, name: &str, kind: TypeKind) {
        let sym = self.interner.intern(name);
        self.type_names.push(sym);
        self.types.push(kind);
    }

    pub fn interner(&self) -> &Interner {
        &self.interner
    }

    pub fn interner_mut(&mut self) -> &mut Interner {
        &mut self.interner
    }

    /// Resolve a type name + generic args to a TypeKind.
    pub fn resolve_type(&self, name: &str, generics: &[TypeKind]) -> Option<TypeKind> {
        TypeKind::from_name(name, generics)
    }

    /// Look up a primitive type by name symbol.
    pub fn lookup_primitive(&mut self, name: &str) -> Option<TypeId> {
        let sym = self.interner.intern(name);
        self.type_names
            .iter()
            .position(|&s| s == sym)
            .map(|i| TypeId(i as u32))
    }

    pub fn type_by_id(&self, id: TypeId) -> Option<&TypeKind> {
        self.types.get(id.0 as usize)
    }

    pub fn lookup_builtin(&self, name: &str) -> Option<&BuiltinSig> {
        BuiltinNode::from_name(name).and_then(|n| {
            self.builtins.iter().find(|b| b.node == n)
        })
    }

    pub fn is_builtin(&self, name: &str) -> bool {
        BuiltinNode::from_name(name).is_some()
    }

    pub fn primitive_types(&self) -> &[(Symbol, TypeKind)] {
        // Not storing pairs; callers use lookup_primitive
        &[]
    }

    /// All registered type kinds.
    pub fn types(&self) -> &[TypeKind] {
        &self.types
    }

    pub fn builtins(&self) -> &[BuiltinSig] {
        &self.builtins
    }
}

impl Default for StlRegistry {
    fn default() -> Self {
        Self::new()
    }
}
