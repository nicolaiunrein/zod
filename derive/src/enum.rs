use crate::docs::RustDocs;
use crate::field::Field;
use darling::ast::Fields;
use darling::FromVariant;
use proc_macro2::TokenStream;
use quote::quote;
use serde_derive_internals::ast::Container;

#[derive(FromVariant, Clone)]
pub struct EnumVariant {
    pub ident: syn::Ident,
    pub fields: Fields<Field>,
}

pub struct Enum<'a> {
    pub(crate) variants: Vec<EnumVariant>,
    pub(crate) container: &'a Container<'a>,
    pub(crate) docs: &'a RustDocs,
}

impl<'a> Enum<'a> {
    pub fn expand(self) -> TokenStream {
        quote!()
    }
}
