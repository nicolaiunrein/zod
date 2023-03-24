use darling::FromAttributes;
use serde_derive_internals::attr::Container;
use syn::{Attribute, Type};

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

pub struct ContainerConfig {
    pub docs: RustDocs,
    pub name: String,
    pub transparent: bool,
    pub from_into: Option<Type>,
    pub namespace: syn::Path,
    pub tag: TagType,
}

#[cfg(test)]
impl Default for ContainerConfig {
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

impl ContainerConfig {
    pub fn new(
        serde_attrs: &Container,
        orig: &[Attribute],
        namespace: syn::Path,
    ) -> Result<Self, darling::Error> {
        let docs = RustDocs::from_attributes(&orig).unwrap();

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
