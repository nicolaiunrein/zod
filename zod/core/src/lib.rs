mod build_ins;

#[cfg(debug_assertions)]
pub mod docs;

use std::collections::BTreeMap;

pub use build_ins::*;

pub trait ZodType {
    fn schema() -> String;
    fn type_def() -> TsTypeDef;

    fn docs() -> Option<&'static str> {
        None
    }

    fn type_name() -> String {
        match Self::type_def() {
            TsTypeDef::Interface(inner) => inner,
            TsTypeDef::Type(inner) => inner,
        }
    }
}

pub trait Namespace {
    const NAME: &'static str;

    fn docs() -> Option<&'static str> {
        None
    }

    #[cfg(feature = "inventory")]
    fn members() -> Vec<&'static NamespaceMemberDefinition> {
        let members = inventory_crate::iter::<NamespaceMemberDefinition>()
            .filter(|namespace| namespace.ns_name == Self::NAME);

        members.collect()
    }
}

type RuntimeValue<T> = &'static (dyn Fn() -> T + Sync);

pub struct NamespaceMemberDefinition {
    ns_name: &'static str,
    name: &'static str,
    schema: RuntimeValue<String>,
    type_def: RuntimeValue<TsTypeDef>,
}

impl NamespaceMemberDefinition {
    #[doc(hidden)]
    pub const fn new_for<T: ZodType + 'static>(ns_name: &'static str, name: &'static str) -> Self {
        Self {
            ns_name,
            name,
            schema: &<T as ZodType>::schema,
            type_def: &<T as ZodType>::type_def,
        }
    }

    pub fn namespace(&self) -> &'static str {
        self.ns_name
    }
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn schema(&self) -> String {
        (self.schema)()
    }

    pub fn type_def(&self) -> TsTypeDef {
        (self.type_def)()
    }

    pub fn collect() -> BTreeMap<&'static str, Vec<&'static NamespaceMemberDefinition>> {
        let mut out = BTreeMap::<&'static str, Vec<&'static NamespaceMemberDefinition>>::default();
        for def in inventory_crate::iter::<NamespaceMemberDefinition>() {
            out.entry(def.namespace()).or_default().push(def);
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TsTypeDef {
    Interface(String),
    Type(String),
}

impl std::ops::Deref for TsTypeDef {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Type(inner) => inner,
            Self::Interface(inner) => inner,
        }
    }
}

impl std::fmt::Display for TsTypeDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TsTypeDef::Interface(s) => write!(f, "{s}"),
            TsTypeDef::Type(s) => write!(f, "{s}"),
        }
    }
}

impl std::cmp::PartialEq<&str> for TsTypeDef {
    fn eq(&self, other: &&str) -> bool {
        let s: &str = self;
        (&s).eq(other)
    }
}

impl std::cmp::PartialEq<str> for TsTypeDef {
    fn eq(&self, other: &str) -> bool {
        let s: &str = self;
        s.eq(other)
    }
}

impl std::cmp::PartialEq<String> for TsTypeDef {
    fn eq(&self, other: &String) -> bool {
        self.eq(other.as_str())
    }
}

inventory_crate::collect!(NamespaceMemberDefinition);

pub trait TypeRegister<T> {}
