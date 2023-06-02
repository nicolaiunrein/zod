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

pub trait FieldAttrsExt {
    fn skip(&self, derive: Derive) -> bool;
    fn is_optional(&self, derive: Derive) -> bool;
}

impl FieldAttrsExt for serde_derive_internals::attr::Field {
    fn skip(&self, derive: Derive) -> bool {
        match derive {
            Derive::Input => self.skip_deserializing(),
            Derive::Output => self.skip_serializing(),
        }
    }

    fn is_optional(&self, derive: Derive) -> bool {
        match derive {
            Derive::Input => !self.default().is_none(),
            Derive::Output => !self.default().is_none() || self.skip_serializing_if().is_some(),
        }
    }
}

pub(crate) trait NameExt {
    fn as_str(&self, derive: Derive) -> String;
}

impl NameExt for serde_derive_internals::attr::Name {
    fn as_str(&self, derive: Derive) -> String {
        match derive {
            Derive::Input => self.deserialize_name(),
            Derive::Output => self.serialize_name(),
        }
    }
}
