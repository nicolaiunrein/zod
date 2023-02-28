use super::field;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use serde_derive_internals::{ast, attr::TagType};
use syn::Ident;

/// represents a struct variant of an enum, it has one or more named fields. It is represeneted as
/// an object in typescript.
pub struct StructVariant<'a> {
    pub ident: &'a Ident,
    pub fields: field::VariantFields<'a>,
    pub serde_ast: &'a ast::Container<'a>,
    pub attrs: &'a serde_derive_internals::attr::Variant,
}

impl<'a> StructVariant<'a> {
    pub fn expand_schema(&self) -> TokenStream {
        let inner = match self.fields.len() {
            0 => unreachable!("Empty tuple structs are handled by darling"),
            1 => {
                let inner = self.fields.expand_schema();
                inner.first().expect("one field").clone()
            }

            _ => {
                let inner = self.fields.expand_schema();
                quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#inner),*];
                        inner.join(", ")
                    }
                }
            }
        };

        let span = self.ident.span();
        let name = self.attrs.name().deserialize_name();

        match self.serde_ast.attrs.tag() {
            TagType::External => {
                quote_spanned! {span =>  format!("z.object({{ {}: z.object({{ {} }}) }})", #name, #inner) }
            }
            TagType::Internal { tag } => {
                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {} }})", #tag, #name, #inner) }
            }
            TagType::Adjacent { tag, content } => {
                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {}: z.object({{ {} }}) }})", #tag, #name, #content, #inner) }
            }
            TagType::None => {
                quote_spanned! {span =>  format!("z.object({{ {} }})", #inner) }
            }
        }
    }

    pub fn expand_type_defs(&self) -> TokenStream {
        let expanded_fields = self.fields.expand_type_defs();
        let span = self.ident.span();
        let name = self.attrs.name().deserialize_name();

        let inner = match expanded_fields.len() {
            0 => unreachable!("Empty tuple structs are handled by darling"),
            1 => expanded_fields
                .first()
                .expect("exactly one variant")
                .clone(),

            _ => {
                quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#expanded_fields),*];
                        inner.join(", ")
                    }
                }
            }
        };

        match self.serde_ast.attrs.tag() {
            // `A{ num: usize, s: String }` -> `{ A: { num: number, s: string } }`
            TagType::External => {
                quote_spanned! {span =>  format!("{{ {}: {{ {} }} }}", #name, #inner) }
            }
            // `A{ num: usize, s: String }` -> `{ type: "A", num: number, s: string }`
            TagType::Internal { tag } => {
                quote_spanned! {span =>  format!("{{ {}: \"{}\", {} }}", #tag, #name, #inner) }
            }

            // `A{ num: usize, s: String }` -> `{ type: "A", content: { num: number, s: string }}`
            TagType::Adjacent { tag, content } => {
                quote_spanned! {span =>  format!("{{ {}: \"{}\", {}: {{ {} }} }}", #tag, #name, #content, #inner) }
            }
            // `A{ num: usize, s: String }` -> `{ num: number, s: string }`
            TagType::None => {
                quote_spanned! {span =>  format!("{{ {} }}", #inner) }
            }
        }
    }
}
