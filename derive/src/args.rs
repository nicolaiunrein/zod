use std::ops::Deref;

use darling::{
    ast::{Data, Fields},
    FromDeriveInput, FromField, FromGenerics, FromVariant,
};
use syn::{Attribute, Ident, Type};

#[derive(FromDeriveInput)]
#[darling(
    attributes(zod),
    forward_attrs(allow, doc, cfg),
    supports(struct_named, struct_newtype, struct_tuple, enum_any)
)]
pub struct Input {
    pub ident: syn::Ident,
    pub data: Data<EnumVariant, StructField>,
    pub namespace: syn::Path,
    pub attrs: Vec<Attribute>,
    pub generics: InputGenerics,
}

#[derive(FromVariant, Clone)]
pub struct EnumVariant {
    pub ident: syn::Ident,
    pub fields: Fields<EnumField>,
}

#[derive(FromField, Clone)]
pub struct StructField {
    pub ident: Option<syn::Ident>,
    pub ty: Type,
}

#[derive(FromField, Clone)]
pub struct EnumField {
    pub ident: Option<syn::Ident>,
    pub ty: Type,
}

#[derive(FromDeriveInput)]
#[darling(attributes(namespace), forward_attrs(allow, doc, cfg))]
pub struct NamespaceInput {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub data: Data<EnumVariant, StructField>,
    pub name: Option<String>,
    pub attrs: Vec<Attribute>,
    pub generics: syn::Generics,
}

#[derive(Clone, Debug)]
pub enum InputGeneric {
    Ident(Ident),
    Lifetime,
}

#[derive(Clone, Debug)]
pub struct InputGenerics {
    pub params: Vec<InputGeneric>,
    orig: syn::Generics,
}

impl Deref for InputGenerics {
    type Target = syn::Generics;

    fn deref(&self) -> &Self::Target {
        &self.orig
    }
}

impl FromGenerics for InputGenerics {
    fn from_generics(generics: &syn::Generics) -> darling::Result<Self> {
        let params = generics
            .params
            .iter()
            .filter_map(|p| match p {
                syn::GenericParam::Type(t) => {
                    if !t.bounds.is_empty() {
                        Some(Err(darling::Error::custom(
                            "zod: `generics with bounds are not supported`",
                        )
                        .with_span(&t.bounds)))
                    } else {
                        Some(Ok(InputGeneric::Ident(t.ident.clone())))
                    }
                }
                syn::GenericParam::Lifetime(_) => Some(Ok(InputGeneric::Lifetime)),
                syn::GenericParam::Const(c) => Some(Err(darling::Error::custom(
                    "zod: `const generics are not supported`",
                )
                .with_span(c))),
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            params,
            orig: generics.clone(),
        })
    }
}
