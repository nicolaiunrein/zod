use super::field;
use super::UnitVariant;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use serde_derive_internals::attr::TagType;

/// represents a tuple variant of an enum, it has one or more unnamed fields. It is represented as a tuple in
/// zod which is a const array in typescript
pub struct TupleVariant<'a> {
    pub span: Span,
    pub tag: &'a TagType,
    pub name: String,
    pub fields: field::VariantFields<'a>,
}

impl<'a> TupleVariant<'a> {
    pub fn expand_schema(&self) -> TokenStream {
        let inner = match self.fields.len() {
            0 => {
                // may occur if fields are skipped. In this case we handle it like a unit variant
                return UnitVariant {
                    span: self.span,
                    tag: self.tag,
                    name: self.name.clone(),
                }
                .expand_schema();
            }
            1 => self
                .fields
                .expand_schema()
                .into_iter()
                .next()
                .expect("one field"),

            _ => {
                let inner = self.fields.expand_schema();
                quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#inner),*];
                        format!("z.tuple([{}])", inner.join(", "))
                    }
                }
            }
        };
        let name = &self.name;
        let span = self.span;

        match self.tag {
            TagType::External | TagType::Internal { .. } => {
                quote_spanned! {span =>  format!("z.object({{ {}: {} }})", #name, #inner) }
            }
            TagType::Adjacent { tag, content } => {
                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {}: {} }})", #tag, #name, #content, #inner) }
            }
            TagType::None => {
                quote_spanned! {span =>  String::from(#inner) }
            }
        }
    }

    pub fn expand_type_defs(&self) -> TokenStream {
        let expanded_fields = self.fields.expand_type_defs();
        let span = self.span;
        let tag_type = self.tag;
        let name = &self.name;

        let inner = match expanded_fields.len() {
            // may occur if fields are skipped. In this case we handle it like a unit variant
            0 => {
                return UnitVariant {
                    span: self.span,
                    tag: self.tag,
                    name: self.name.clone(),
                }
                .expand_type_defs()
            }
            1 => expanded_fields
                .first()
                .expect("exactly one variant")
                .clone(),

            _ => {
                quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#expanded_fields),*];
                        format!("[{}]", inner.join(", "))
                    }
                }
            }
        };

        match tag_type {
            TagType::External | TagType::Internal { .. } => {
                quote_spanned! {span =>  format!("{{ {}: {} }}", #name, #inner) }
            }
            TagType::Adjacent { tag, content } => {
                quote_spanned! {span =>  format!("{{ {}: \"{}\", {}: {} }}", #tag, #name, #content, #inner) }
            }
            TagType::None => {
                quote_spanned! {span =>  #inner }
            }
        }
    }
}
