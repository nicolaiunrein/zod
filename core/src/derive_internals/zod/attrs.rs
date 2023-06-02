use darling::FromDeriveInput;

use super::Derive;

#[derive(FromDeriveInput)]
#[darling(
    attributes(zod),
    forward_attrs(allow, doc, cfg),
    supports(struct_named, struct_newtype, struct_tuple, enum_any)
)]

pub(super) struct ZodAttrs {
    pub namespace: syn::Path,
    pub custom_suffix: Option<String>,
}

pub(crate) trait NameExt {
    fn as_str(&self, derive: Derive) -> String;
}

pub(crate) trait DefaultExt {
    fn is_optional(&self) -> bool;
}

impl NameExt for serde_derive_internals::attr::Name {
    fn as_str(&self, derive: Derive) -> String {
        match derive {
            Derive::Input => self.deserialize_name(),
            Derive::Output => self.serialize_name(),
        }
    }
}

impl DefaultExt for serde_derive_internals::attr::Default {
    fn is_optional(&self) -> bool {
        match self {
            serde_derive_internals::attr::Default::Default
            | serde_derive_internals::attr::Default::Path(_) => true,
            serde_derive_internals::attr::Default::None => false,
        }
    }
}
