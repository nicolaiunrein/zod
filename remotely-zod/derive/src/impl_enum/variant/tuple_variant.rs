use super::field;
use super::UnitVariant;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use serde_derive_internals::{ast, attr::TagType};
use syn::Ident;

/// represents a tuple variant of an enum, it has one or more unnamed fields. It is represented as a tuple in
/// zod which is a const array in typescript
pub struct TupleVariant<'a> {
    pub ident: &'a Ident,
    pub fields: field::VariantFields<'a>,
    pub serde_ast: &'a ast::Container<'a>,
    pub attrs: &'a serde_derive_internals::attr::Variant,
}

impl<'a> TupleVariant<'a> {
    pub fn expand_schema(&self) -> TokenStream {
        match self.fields.len() {
            // may occur if fields are skipped. In this case we handle it like a unit variant
            0 => UnitVariant {
                ident: self.ident,
                serde_ast: self.serde_ast,
                attrs: self.attrs,
            }
            .expand_schema(),
            1 => self.expand_one_schema(),
            _ => self.expand_n_schemas(),
        }
    }

    pub fn expand_type_defs(&self) -> TokenStream {
        let expanded_fields = self.fields.expand_type_defs();
        let span = self.ident.span();
        let tag_type = self.serde_ast.attrs.tag();
        let name = self.attrs.name().deserialize_name();

        match expanded_fields.len() {
            // may occur if fields are skipped. In this case we handle it like a unit variant
            0 => UnitVariant {
                ident: self.ident,
                serde_ast: self.serde_ast,
                attrs: self.attrs,
            }
            .expand_type_defs(),
            1 => {
                let first = expanded_fields.first().expect("exactly one variant");

                // expand an enum variant with exatly one field to a TS definition
                // External: `A(usize)` ->  `{ A: number }`
                // Adjacent: `A(usize)` ->  `{ type: "A", content: number }`
                match tag_type {
                    TagType::External | TagType::Internal { .. } => {
                        quote_spanned! {span =>  format!("{{ {}: {} }}", #name, #first) }
                    }
                    TagType::Adjacent { tag, content } => {
                        quote_spanned! {span =>  format!("{{ {}: \"{}\", {}: {} }}", #tag, #name, #content, #first) }
                    }
                    TagType::None => {
                        quote_spanned! {span =>  String::from(#first) }
                    }
                }
            }

            // expand an enum tuple variant with more than one field to a TS definition
            // Example
            // `A(usize, String)` -> `{ A: [number, string] }`
            _ => {
                let expanded_inner = quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#expanded_fields),*];
                        format!("[{}]", inner.join(", "))
                    }
                };

                match tag_type {
                    TagType::External | TagType::Internal { .. } => {
                        quote_spanned! {span =>  format!("{{ {}: {} }}", #name, #expanded_inner) }
                    }
                    TagType::Adjacent { tag, content } => {
                        quote_spanned! {span =>  format!("{{ {}: \"{}\", {}: {} }}", #tag, #name, #content, #expanded_inner) }
                    }
                    TagType::None => {
                        quote_spanned! {span => #expanded_inner }
                    }
                }
            }
        }
    }

    /// expand an enum variant with exatly one field into a zod schema
    /// Extern: `A(usize)  =>  z.object({ A: z.number().int().nonnegative() })`
    /// Intern: Unsupported
    /// Adjacent: `A(usize)  =>  z.object({ type: "A", content: z.number().int().nonnegative() })`
    /// Untagged: `A(usize)  =>  z.number().int().nonnegative()`
    fn expand_one_schema(&self) -> TokenStream {
        let inner = self.fields.expand_schema();
        let first = inner.first().unwrap();
        let name = self.attrs.name().deserialize_name();
        let span = self.ident.span();
        match self.serde_ast.attrs.tag() {
            TagType::External | TagType::Internal { .. } => {
                quote_spanned! {span =>  format!("z.object({{ {}: {} }})", #name, #first) }
            }
            TagType::Adjacent { tag, content } => {
                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {}: {} }})", #tag, #name, #content, #first) }
            }
            TagType::None => {
                quote_spanned! {span =>  String::from(#first) }
            }
        }
    }

    /// expand an enum tuple variant with more than one field into a zod schema
    /// Example: `A(usize, String)`  ->
    /// `z.object({ A: z.tuple([z.number().int().nonnegative(),  z.string()]) })`
    fn expand_n_schemas(&self) -> TokenStream {
        let inner = self.fields.expand_schema();
        let span = self.ident.span();
        let name = self.attrs.name().deserialize_name();

        let expanded_inner = quote! {
            {
                let inner: std::vec::Vec<String> = vec![#(#inner),*];
                format!("z.tuple([{}])", inner.join(", "))
            }
        };
        match self.serde_ast.attrs.tag() {
            TagType::External | TagType::Internal { .. } => {
                quote_spanned! {span =>  format!("z.object({{ {}: {} }})", #name, #expanded_inner) }
            }
            TagType::Adjacent { tag, content } => {
                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {}: {} }})", #tag, #name, #content, #expanded_inner) }
            }
            TagType::None => {
                quote_spanned! {span =>  String::from(#expanded_inner) }
            }
        }
    }
}
