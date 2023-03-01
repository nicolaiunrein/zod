mod build_ins;

use std::collections::BTreeMap;

pub use build_ins::*;

pub trait ZodType {
    fn schema() -> String;
    fn type_def() -> String;

    fn docs() -> Option<&'static str> {
        None
    }
    fn type_name() -> String {
        Self::type_def()
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
    type_def: RuntimeValue<String>,
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

    pub fn type_def(&self) -> String {
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

inventory_crate::collect!(NamespaceMemberDefinition);

pub trait TypeRegister<T> {}
