#![allow(dead_code)]
use std::collections::HashMap;
use std::rc::{Rc, Weak};

pub use crate::parser::BuiltinType;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(String);

impl Id {
    pub fn new<S: AsRef<str>>(s: S) -> Self {
        Id(s.as_ref().to_string())
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    definitions: Vec<Definition>,
    entries: HashMap<Id, Entry>,
}

impl Document {
    pub(crate) fn new(definitions: Vec<Definition>, entries: HashMap<Id, Entry>) -> Self {
        Document {
            definitions,
            entries,
        }
    }
    pub fn typename(&self, name: &Id) -> Option<Rc<NamedType>> {
        self.entries.get(name).and_then(|e| match e {
            Entry::Typename(nt) => Some(nt.upgrade().expect("always possible to upgrade entry")),
            _ => None,
        })
    }
    pub fn typenames<'a>(&'a self) -> impl Iterator<Item = Rc<NamedType>> + 'a {
        self.definitions.iter().filter_map(|d| match d {
            Definition::Typename(nt) => Some(nt.clone()),
            _ => None,
        })
    }
    pub fn module(&self, name: &Id) -> Option<Rc<Module>> {
        self.entries.get(&name).and_then(|e| match e {
            Entry::Module(m) => Some(m.upgrade().expect("always possible to upgrade entry")),
            _ => None,
        })
    }
    pub fn modules<'a>(&'a self) -> impl Iterator<Item = Rc<Module>> + 'a {
        self.definitions.iter().filter_map(|d| match d {
            Definition::Module(m) => Some(m.clone()),
            _ => None,
        })
    }
}

impl PartialEq for Document {
    fn eq(&self, rhs: &Document) -> bool {
        // For equality, we don't care about the ordering of definitions,
        // so we only need to check that the entries map is equal
        self.entries == rhs.entries
    }
}
impl Eq for Document {}

#[derive(Debug, Clone)]
pub enum Definition {
    Typename(Rc<NamedType>),
    Module(Rc<Module>),
}

#[derive(Debug, Clone)]
pub enum Entry {
    Typename(Weak<NamedType>),
    Module(Weak<Module>),
}

impl Entry {
    pub fn kind(&self) -> &'static str {
        match self {
            Entry::Typename { .. } => "typename",
            Entry::Module { .. } => "module",
        }
    }
}

impl PartialEq for Entry {
    fn eq(&self, rhs: &Entry) -> bool {
        match (self, rhs) {
            (Entry::Typename(t), Entry::Typename(t_rhs)) => {
                t.upgrade()
                    .expect("possible to upgrade entry when part of document")
                    == t_rhs
                        .upgrade()
                        .expect("possible to upgrade entry when part of document")
            }
            (Entry::Module(m), Entry::Module(m_rhs)) => {
                m.upgrade()
                    .expect("possible to upgrade entry when part of document")
                    == m_rhs
                        .upgrade()
                        .expect("possible to upgrade entry when part of document")
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeRef {
    Name(Rc<NamedType>),
    Value(Rc<Type>),
}

impl TypeRef {
    pub fn type_(&self) -> Rc<Type> {
        match self {
            TypeRef::Name(named) => named.type_(),
            TypeRef::Value(ref v) => v.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamedType {
    pub name: Id,
    pub dt: TypeRef,
    pub docs: String,
}

impl NamedType {
    pub fn type_(&self) -> Rc<Type> {
        self.dt.type_()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Enum(EnumDatatype),
    Flags(FlagsDatatype),
    Struct(StructDatatype),
    Union(UnionDatatype),
    Handle(HandleDatatype),
    Array(TypeRef),
    Pointer(TypeRef),
    ConstPointer(TypeRef),
    Builtin(BuiltinType),
}

impl Type {
    pub fn kind(&self) -> &'static str {
        use Type::*;
        match self {
            Enum(_) => "enum",
            Flags(_) => "flags",
            Struct(_) => "struct",
            Union(_) => "union",
            Handle(_) => "handle",
            Array(_) => "array",
            Pointer(_) => "pointer",
            ConstPointer(_) => "constpointer",
            Builtin(_) => "builtin",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IntRepr {
    U8,
    U16,
    U32,
    U64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumDatatype {
    pub repr: IntRepr,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariant {
    pub name: Id,
    pub docs: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlagsDatatype {
    pub repr: IntRepr,
    pub flags: Vec<FlagsMember>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlagsMember {
    pub name: Id,
    pub docs: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDatatype {
    pub members: Vec<StructMember>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructMember {
    pub name: Id,
    pub tref: TypeRef,
    pub docs: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionDatatype {
    pub variants: Vec<UnionVariant>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnionVariant {
    pub name: Id,
    pub tref: TypeRef,
    pub docs: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleDatatype {
    pub supertypes: Vec<TypeRef>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: Id,
    definitions: Vec<ModuleDefinition>,
    entries: HashMap<Id, ModuleEntry>,
    pub docs: String,
}

impl Module {
    pub(crate) fn new(
        name: Id,
        definitions: Vec<ModuleDefinition>,
        entries: HashMap<Id, ModuleEntry>,
        docs: String,
    ) -> Self {
        Module {
            name,
            definitions,
            entries,
            docs,
        }
    }
    pub fn import(&self, name: &Id) -> Option<Rc<ModuleImport>> {
        self.entries.get(name).and_then(|e| match e {
            ModuleEntry::Import(d) => Some(d.upgrade().expect("always possible to upgrade entry")),
            _ => None,
        })
    }
    pub fn imports<'a>(&'a self) -> impl Iterator<Item = Rc<ModuleImport>> + 'a {
        self.definitions.iter().filter_map(|d| match d {
            ModuleDefinition::Import(d) => Some(d.clone()),
            _ => None,
        })
    }
    pub fn func(&self, name: &Id) -> Option<Rc<InterfaceFunc>> {
        self.entries.get(name).and_then(|e| match e {
            ModuleEntry::Func(d) => Some(d.upgrade().expect("always possible to upgrade entry")),
            _ => None,
        })
    }
    pub fn funcs<'a>(&'a self) -> impl Iterator<Item = Rc<InterfaceFunc>> + 'a {
        self.definitions.iter().filter_map(|d| match d {
            ModuleDefinition::Func(d) => Some(d.clone()),
            _ => None,
        })
    }
}

impl PartialEq for Module {
    fn eq(&self, rhs: &Module) -> bool {
        // For equality, we don't care about the ordering of definitions,
        // so we only need to check that the entries map is equal
        self.entries == rhs.entries
    }
}
impl Eq for Module {}

#[derive(Debug, Clone)]
pub enum ModuleDefinition {
    Import(Rc<ModuleImport>),
    Func(Rc<InterfaceFunc>),
}

#[derive(Debug, Clone)]
pub enum ModuleEntry {
    Import(Weak<ModuleImport>),
    Func(Weak<InterfaceFunc>),
}

impl PartialEq for ModuleEntry {
    fn eq(&self, rhs: &ModuleEntry) -> bool {
        match (self, rhs) {
            (ModuleEntry::Import(i), ModuleEntry::Import(i_rhs)) => {
                i.upgrade()
                    .expect("always possible to upgrade moduleentry when part of module")
                    == i_rhs
                        .upgrade()
                        .expect("always possible to upgrade moduleentry when part of module")
            }
            (ModuleEntry::Func(i), ModuleEntry::Func(i_rhs)) => {
                i.upgrade()
                    .expect("always possible to upgrade moduleentry when part of module")
                    == i_rhs
                        .upgrade()
                        .expect("always possible to upgrade moduleentry when part of module")
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleImport {
    pub name: Id,
    pub variant: ModuleImportVariant,
    pub docs: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleImportVariant {
    Memory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceFunc {
    pub name: Id,
    pub params: Vec<InterfaceFuncParam>,
    pub results: Vec<InterfaceFuncParam>,
    pub docs: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceFuncParam {
    pub name: Id,
    pub tref: TypeRef,
    pub position: InterfaceFuncParamPosition,
    pub docs: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterfaceFuncParamPosition {
    Param(usize),
    Result(usize),
}
