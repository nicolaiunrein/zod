use darling::FromAttributes;
use serde_derive_internals::attr::Container;
use syn::{Attribute, Type};

use crate::docs::RustDocs;

use super::Derive;

#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) enum TagType {
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
            serde_derive_internals::attr::TagType::Internal { tag } => TagType::Internal { tag },
            serde_derive_internals::attr::TagType::Adjacent { tag, content } => {
                TagType::Adjacent { tag, content }
            }
            serde_derive_internals::attr::TagType::None => TagType::None,
        }
    }
}

pub(crate) struct ContainerConfig {
    pub(crate) docs: RustDocs,
    pub(crate) name: String,
    pub(crate) transparent: bool,
    pub(crate) type_alias: Option<Type>,
    pub(crate) namespace: syn::Path,
    pub(crate) tag: TagType,
    pub(crate) derive: Derive,
}

#[cfg(test)]
impl Default for ContainerConfig {
    fn default() -> Self {
        Self {
            docs: Default::default(),
            name: String::from("MyType"),
            transparent: false,
            type_alias: None,
            namespace: syn::parse_quote!(Ns),
            tag: Default::default(),
            derive: Derive::Request,
        }
    }
}

impl ContainerConfig {
    pub(crate) fn new(
        serde_attrs: &Container,
        orig: &[Attribute],
        namespace: syn::Path,
        derive: Derive,
    ) -> Result<Self, darling::Error> {
        let docs = RustDocs::from_attributes(orig).unwrap();

        let name = match derive {
            Derive::Request => serde_attrs.name().deserialize_name(),
            Derive::Response => serde_attrs.name().serialize_name(),
        };

        let transparent = serde_attrs.transparent();

        let type_alias = match derive {
            Derive::Request => serde_attrs
                .type_from()
                .or_else(|| serde_attrs.type_try_from()),
            Derive::Response => serde_attrs.type_into(),
        }
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
            type_alias,
            namespace,
            tag,
            derive,
        })
    }
}
