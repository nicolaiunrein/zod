use darling::{FromAttributes, FromDeriveInput};
use syn::{Expr, Lit, Meta};

use super::{fields::FieldValue, Derive};

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

#[derive(FromAttributes)]
#[darling(attributes(zod))]
pub(super) struct ZodFieldAttrs {
    override_input_with: Option<syn::Path>,
    override_output_with: Option<syn::Path>,
    override_with: Option<syn::Path>,
}

impl ZodFieldAttrs {
    pub(super) fn as_field_value(&self, derive: Derive) -> Option<FieldValue> {
        let fail = || match derive {
            Derive::Input => {
                r#"only one of #[zod(override_with = "...")] or #[zod(override_input_with = "...")] may be specified"#
            }
            Derive::Output => {
                r#"only one of #[zod(override_with = "...")] or #[zod(override_output_with = "...")] may be specified"#
            }
        };
        match derive {
            Derive::Input => match (&self.override_with, &self.override_input_with) {
                (None, None) => None,
                (None, Some(p)) | (Some(p), None) => Some(FieldValue::OverrideGetter(p.clone())),
                _ => panic!("{}", fail()),
            },
            Derive::Output => match (&self.override_with, &self.override_output_with) {
                (None, None) => None,
                (None, Some(p)) | (Some(p), None) => Some(FieldValue::OverrideGetter(p.clone())),
                _ => panic!("{}", fail()),
            },
        }
    }
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

pub fn get_rustdoc(attrs: &[syn::Attribute]) -> Option<String> {
    let mut full_docs = String::new();
    for attr in attrs {
        match attr.meta {
            Meta::NameValue(ref nv) => {
                if nv.path.is_ident("doc") {
                    match nv.value {
                        Expr::Lit(ref lit_expr) => match lit_expr.lit {
                            Lit::Str(ref s) => {
                                full_docs.push_str(s.value().trim());
                                full_docs.push('\n');
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    if full_docs.is_empty() {
        None
    } else {
        Some(full_docs)
    }
}
