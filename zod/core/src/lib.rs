mod build_ins;

pub use build_ins::*;

pub trait ZodType {
    fn schema() -> String;
    fn docs() -> Option<&'static str> {
        None
    }
    fn type_def() -> String;
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
}

inventory_crate::collect!(NamespaceMemberDefinition);

pub trait TypeRegister<T> {}
