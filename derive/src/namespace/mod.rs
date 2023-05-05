use crate::docs::RustDocs;
use crate::utils::get_zod;
use darling::{FromAttributes, FromDeriveInput, ToTokens};
use quote::quote;

#[derive(FromDeriveInput)]
#[darling(attributes(namespace), forward_attrs(allow, doc, cfg))]
pub(crate) struct Namespace {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub name: Option<String>,
    pub generics: syn::Generics,
    pub attrs: Vec<syn::Attribute>,
    pub doc: Option<String>,
}

impl ToTokens for Namespace {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.ident;

        let name = &self
            .name
            .as_ref()
            .cloned()
            .unwrap_or_else(|| ident.to_string());

        let docs = RustDocs::from_attributes(&self.attrs).unwrap();

        let zod = get_zod();

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let expanded = quote! {
            impl #impl_generics #zod::core::Namespace for #ident #ty_generics #where_clause {
                const NAME: &'static str = #name;
                const DOCS: Option<#zod::core::ast::Docs> = #docs;
            }
        };

        tokens.extend(expanded)
    }
}
