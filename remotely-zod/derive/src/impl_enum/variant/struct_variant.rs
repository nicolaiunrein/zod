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
        match self.fields.len() {
            // this case is handled by darling
            0 => unreachable!("5"),
            1 => self.expand_one_field(),
            _ => self.expand_many_fields(),
        }
    }

    /// expand an enum variant with exatly one field into a zod schema
    /// External: `A{ num: usize } =>  z.object({ A: z.object({ num: z.number().int().nonnegative() }) })`
    /// Internal: `A{ num: usize } =>  z.object({ type: z.literal("A"), num: z.number().int().nonnegative() })`
    /// Adjacent: `A{ num: usize } =>  z.object({ type: z.literal("A"), content: z.object({ num: z.number().int().nonnegative() }) })`
    fn expand_one_field(&self) -> TokenStream {
        let inner = self.fields.expand_schema();
        let name = self.attrs.name().deserialize_name();

        let span = self.ident.span();
        let first = inner.first().unwrap();
        // let name = variant_names.first().unwrap();

        match self.serde_ast.attrs.tag() {
            TagType::External => {
                quote_spanned! {span =>  format!("z.object({{ {}: z.object({{ {} }}) }})", #name, #first) }
            }
            TagType::Internal { tag } => {
                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {} }})", #tag, #name, #first) }
            }
            TagType::Adjacent { tag, content } => {
                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {}: z.object({{ {} }}) }})", #tag, #name, #content, #first) }
            }
            TagType::None => {
                quote_spanned! {span =>  format!("z.object({{ {} }})", #first) }
            }
        }
    }

    pub fn expand_type_defs(&self) -> TokenStream {
        let expanded_fields = self.fields.expand_type_defs();
        let span = self.ident.span();
        let name = self.attrs.name().deserialize_name();

        match expanded_fields.len() {
            // this case is handles by darling
            0 => unreachable!("7"),
            1 => {
                let first = expanded_fields.first().expect("exactly one variant");

                // expand an enum variant with exatly one field to a TS definition
                // External: `A{ num: usize }` ->  `{ A: { num: number }}`
                // Internal: `A{ num: usize }` ->  `{ type: "A", num: number }`
                // Adjacent: `A{ num: usize }` ->  `{ type: "A", content: { num: number }}`
                match self.serde_ast.attrs.tag() {
                    TagType::External => {
                        quote_spanned! {span =>  format!("{{ {}: {{ {} }} }}", #name, #first) }
                    }
                    TagType::Internal { tag } => {
                        quote_spanned! {span =>  format!("{{ {}: \"{}\", {} }}", #tag, #name, #first) }
                    }
                    TagType::Adjacent { tag, content } => {
                        quote_spanned! {span =>  format!("{{ {}: \"{}\", {}: {{ {} }} }}", #tag, #name, #content, #first) }
                    }
                    TagType::None => {
                        quote_spanned! {span =>  format!("{{ {} }}", #first) }
                    }
                }
            }

            // expand an enum tuple variant with more than one field to a TS definition
            // External: `A{ num: usize, s: String }` -> `{ A: { num: number, s: string } }`
            // Internal: `A{ num: usize, s: String }` -> `{ type: "A", num: number, s: string }`
            // Adjacent: `A{ num: usize, s: String }` -> `{ type: "A", content: { num: number, s: string }}`
            _ => {
                let expanded_inner = quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#expanded_fields),*];
                        inner.join(", ")
                    }
                };
                match self.serde_ast.attrs.tag() {
                    TagType::External => {
                        quote_spanned! {span =>  format!("{{ {}: {{ {} }} }}", #name, #expanded_inner) }
                    }
                    TagType::Internal { tag } => {
                        quote_spanned! {span =>  format!("{{ {}: \"{}\", {} }}", #tag, #name, #expanded_inner) }
                    }
                    TagType::Adjacent { tag, content } => {
                        quote_spanned! {span =>  format!("{{ {}: \"{}\", {}: {{ {} }} }}", #tag, #name, #content, #expanded_inner) }
                    }
                    TagType::None => {
                        quote_spanned! {span =>  format!("{{ {} }}", #expanded_inner) }
                    }
                }
            }
        }
    }

    /// expand an enum struct variant with more than one field into a zod schema
    /// External: `A{ num: usize, s: String}` ->
    /// `z.object({ A: z.object({ num: z.number().int().nonnegative(),  s: z.string()}) })`
    ///
    /// Internal: `A{ num: usize, s: String}` ->
    /// `z.object({ type: z.literal("A"), num: z.number().int().nonnegative(), s: z.string()})`
    ///
    /// Adjacent: `A{ num: usize, s: String}` ->
    /// `z.object({ type: z.literal("A"): content: z.object({ num: z.number().int().nonnegative(),  s: z.string()}) })`
    fn expand_many_fields(&self) -> TokenStream {
        let inner = self.fields.expand_schema();
        let span = self.ident.span();
        let name = self.attrs.name().deserialize_name();

        match self.serde_ast.attrs.tag() {
            TagType::External => {
                let expanded_inner = quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#inner),*];
                        inner.join(", ")
                    }
                };
                quote_spanned! {span =>  format!("z.object({{ {}: z.object({{ {} }}) }})", #name, #expanded_inner) }
            }
            TagType::Internal { tag } => {
                let expanded_inner = quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#inner),*];
                        inner.join(", ")
                    }
                };

                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {} }})", #tag, #name, #expanded_inner) }
            }
            TagType::Adjacent { tag, content } => {
                let expanded_inner = quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#inner),*];
                        inner.join(", ")
                    }
                };

                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {}: z.object({{ {} }}) }})", #tag, #name, #content, #expanded_inner) }
            }
            TagType::None => {
                let expanded_inner = quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#inner),*];
                        inner.join(", ")
                    }
                };
                quote_spanned! {span =>  format!("z.object({{ {} }})", #expanded_inner) }
            }
        }
    }
}
