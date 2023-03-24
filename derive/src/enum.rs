use crate::config::Config;
use crate::field::Field;
use darling::ast::Fields;
use darling::FromVariant;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(FromVariant)]
pub struct EnumVariant {
    pub ident: syn::Ident,
    pub fields: Fields<Field>,
}

pub struct Enum<'a> {
    pub(crate) variants: &'a [EnumVariant],
    pub(crate) config: &'a Config,
}

impl<'a> Enum<'a> {
    pub fn expand(self) -> TokenStream {
        quote!()
    }
}
