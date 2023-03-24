use darling::FromAttributes;
use syn::Type;

use crate::docs::RustDocs;
use crate::error::{Error, SerdeConflict};

#[derive(Clone, Debug, PartialEq, Default)]
pub enum TagType {
    #[default]
    None,
    External,
    Internal {
        tag: String,
    },
    Adjacent {
        tag: String,
        content: String,
    },
}

impl From<serde_derive_internals::attr::TagType> for TagType {
    fn from(value: serde_derive_internals::attr::TagType) -> Self {
        match value {
            serde_derive_internals::attr::TagType::External => TagType::External,
            serde_derive_internals::attr::TagType::Internal { tag } => TagType::Internal {
                tag: tag.to_owned(),
            },
            serde_derive_internals::attr::TagType::Adjacent { tag, content } => TagType::Adjacent {
                tag: tag.to_owned(),
                content: content.to_owned(),
            },
            serde_derive_internals::attr::TagType::None => TagType::None,
        }
    }
}

pub struct Config {
    pub docs: RustDocs,
    pub name: String,
    pub transparent: bool,
    pub from_into: Option<Type>,
    pub namespace: syn::Path,
    pub tag: TagType,
}

#[cfg(test)]
impl Default for Config {
    fn default() -> Self {
        Self {
            docs: Default::default(),
            name: String::from("MyType"),
            transparent: false,
            from_into: None,
            namespace: syn::parse_quote!(Ns),
            tag: Default::default(),
        }
    }
}

impl Config {
    pub fn new(orig: &syn::DeriveInput, namespace: syn::Path) -> Result<Self, darling::Error> {
        let cx = serde_derive_internals::Ctxt::new();

        let serde_attrs = serde_derive_internals::ast::Container::from_ast(
            &cx,
            &orig,
            serde_derive_internals::Derive::Deserialize,
        )
        .ok_or_else(|| Error::NoSerde)?
        .attrs;

        if let Err(errors) = cx.check() {
            let mut darling_errors = darling::Error::accumulator();
            for err in errors.into_iter() {
                darling_errors.push(err.into())
            }

            darling_errors.finish()?;
        }

        let docs = RustDocs::from_attributes(&orig.attrs).unwrap();

        let name = {
            let name = serde_attrs.name();
            let ser = name.serialize_name();
            let de = name.deserialize_name();

            if ser != de {
                return Err(Error::SerdeConflict(SerdeConflict::Name { ser, de }).into());
            } else {
                ser
            }
        };

        let transparent = serde_attrs.transparent();

        let from_ty = serde_attrs
            .type_from()
            .or_else(|| serde_attrs.type_try_from());

        let into_ty = serde_attrs.type_into();

        let from_into = match (from_ty, into_ty) {
            (None, None) => Ok(None),
            (Some(from), Some(into)) if from == into => Ok(Some(from)),
            (from, into) => Err(Error::from(SerdeConflict::Type {
                from: from.cloned(),
                into: into.cloned(),
            })),
        }?
        .cloned();

        let tag = match serde_attrs.tag() {
            serde_derive_internals::attr::TagType::External => TagType::External,
            serde_derive_internals::attr::TagType::Internal { tag } => TagType::Internal {
                tag: tag.to_owned(),
            },
            serde_derive_internals::attr::TagType::Adjacent { tag, content } => TagType::Adjacent {
                tag: tag.to_owned(),
                content: content.to_owned(),
            },
            serde_derive_internals::attr::TagType::None => TagType::None,
        };

        Ok(Self {
            docs,
            name,
            transparent,
            from_into,
            namespace,
            tag,
        })
    }
}
