use proc_macro2::Ident;
use quote::ToTokens;
use syn::Generics;

pub(crate) struct EnumImpl<Io> {
    pub(crate) ident: Ident,
    pub(crate) generics: Generics,
    pub(crate) variants: Vec<syn::Variant>,
    pub(crate) kind: Io,
    pub(crate) ns: syn::Path,
    pub(crate) custom_suffix: Option<String>,
}

impl<Io> ToTokens for EnumImpl<Io> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        todo!()
    }
}
