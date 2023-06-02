use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use serde_derive_internals::attr::TagType as SerdeTagType;

use super::r#struct::ZodObjectImpl;
use super::Derive;
use crate::derive_internals::zod::r#struct::ZodTupleImpl;
use crate::utils::zod_core;

#[derive(Default, Clone, Debug, PartialEq)]
pub enum TagType {
    #[default]
    Externally,
    Internally {
        tag: String,
    },
    Adjacently {
        tag: String,
        content: String,
    },
    Untagged,
}

impl From<&SerdeTagType> for TagType {
    fn from(value: &SerdeTagType) -> Self {
        match value {
            SerdeTagType::External => TagType::Externally,
            SerdeTagType::Internal { tag } => TagType::Internally {
                tag: tag.to_owned(),
            },
            SerdeTagType::Adjacent { tag, content } => TagType::Adjacently {
                tag: tag.to_owned(),
                content: content.to_owned(),
            },
            SerdeTagType::None => TagType::Untagged,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct EnumImpl {
    pub(crate) tag: TagType,
    pub(crate) variants: Vec<VariantImpl>,
    pub(crate) derive: Derive,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum VariantImpl {
    Literal(String),
    Object(ZodObjectImpl),
    Tuple(ZodTupleImpl),
}

impl ToTokens for VariantImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            VariantImpl::Literal(name) => {
                quote!(#zod_core::z::ZodLiteral::String(#name).into())
            }
            VariantImpl::Object(obj) => quote!(#obj.into()),
            VariantImpl::Tuple(tuple) => quote!(#tuple.into()),
        }
        .to_tokens(tokens)
    }
}

impl ToTokens for EnumImpl {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variants = &self.variants;
        let out = match &self.tag {
            TagType::Externally | TagType::Untagged => {
                quote! {
                    #zod_core::z::ZodUnion {
                        variants: ::std::vec![#(#variants),*]
                    }
                }
            }
            TagType::Internally { tag } | TagType::Adjacently { tag, .. } => {
                quote! {
                    #zod_core::z::ZodDiscriminatedUnion {
                        tag: #tag,
                        variants: ::std::vec![#(#variants),*]
                    }
                }
            }
        };

        tokens.extend(out)
    }
}
