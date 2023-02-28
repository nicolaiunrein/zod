use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;
use serde_derive_internals::attr::TagType;

/// represents a unit variant of an enum, it has no fields and it is represented in typescript as the
/// stringifyied name
pub struct UnitVariant<'a> {
    pub span: Span,
    pub tag: &'a TagType,
    pub name: String,
}

impl<'a> UnitVariant<'a> {
    pub fn expand_schema(&self) -> TokenStream {
        let name = &self.name;
        match self.tag {
            TagType::External => {
                quote_spanned!(self.span => format!("z.literal(\"{}\")", #name))
            }
            TagType::Internal { tag } => {
                quote_spanned!(self.span=> format!("z.object({{ {}: z.literal(\"{}\") }})", #tag, #name))
            }
            TagType::Adjacent { tag, .. } => {
                quote_spanned!(self.span=> format!("z.object({{ {}: z.literal(\"{}\") }})", #tag, #name))
            }
            TagType::None => {
                quote_spanned!(self.span => String::from("z.null()"))
            }
        }
    }

    /// Example `A`  ->  `"A"`
    pub fn expand_type_defs(&self) -> TokenStream {
        let name = &self.name;
        match self.tag {
            TagType::External => {
                quote_spanned!(self.span => format!("\"{}\"", #name))
            }
            TagType::Internal { tag } => {
                quote_spanned!(self.span => format!("{{ {}: \"{}\" }}", #tag, #name))
            }
            TagType::Adjacent { tag, .. } => {
                quote_spanned!(self.span => format!("{{ {}: \"{}\" }}", #tag, #name))
            }
            TagType::None => {
                quote_spanned!(self.span => String::from("null"))
            }
        }
    }
}
