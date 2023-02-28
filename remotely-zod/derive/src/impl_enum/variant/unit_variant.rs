use proc_macro2::TokenStream;
use quote::quote_spanned;
use serde_derive_internals::{ast, attr::TagType};
use syn::Ident;

/// represents a unit variant of an enum, it has no fields and it is represented in typescript as the
/// stringifyied name
pub struct UnitVariant<'a> {
    pub ident: &'a Ident,
    pub serde_ast: &'a ast::Container<'a>,
    pub attrs: &'a serde_derive_internals::attr::Variant,
}

impl<'a> UnitVariant<'a> {
    pub fn expand_schema(&self) -> TokenStream {
        let name = self.attrs.name().deserialize_name();
        match self.serde_ast.attrs.tag() {
            TagType::External => {
                quote_spanned!(self.ident.span() => format!("z.literal(\"{}\")", #name))
            }
            TagType::Internal { tag } => {
                quote_spanned!(self.ident.span() => format!("z.object({{ {}: z.literal(\"{}\") }})", #tag, #name))
            }
            TagType::Adjacent { tag, .. } => {
                quote_spanned!(self.ident.span() => format!("z.object({{ {}: z.literal(\"{}\") }})", #tag, #name))
            }
            TagType::None => {
                quote_spanned!(self.ident.span() => String::from("z.null()"))
            }
        }
    }

    /// Example `A`  ->  `"A"`
    pub fn expand_type_defs(&self) -> TokenStream {
        let name = self.attrs.name().deserialize_name();
        let span = self.ident.span();
        match self.serde_ast.attrs.tag() {
            TagType::External => {
                quote_spanned!(span => format!("\"{}\"", #name))
            }
            TagType::Internal { tag } => {
                quote_spanned!(span => format!("{{ {}: \"{}\" }}", #tag, #name))
            }
            TagType::Adjacent { tag, .. } => {
                quote_spanned!(span => format!("{{ {}: \"{}\" }}", #tag, #name))
            }
            TagType::None => {
                quote_spanned!(span => String::from("null"))
            }
        }
    }
}
