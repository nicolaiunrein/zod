use proc_macro2::Ident;
use quote::ToTokens;
use syn::Generics;

pub(crate) struct EnumImpl<Io> {
    pub(crate) variants: Vec<syn::Variant>,
    pub(crate) kind: Io,
}

impl<Io> ToTokens for EnumImpl<Io> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        todo!()
    }
}
