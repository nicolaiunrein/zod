mod struct_variant;
mod tuple_variant;
mod unit_variant;

use struct_variant::StructVariant;
use tuple_variant::TupleVariant;
use unit_variant::UnitVariant;

use super::args;
use super::field;
use darling::ast::Style;
use proc_macro2::TokenStream;
use serde_derive_internals::ast;

/// represents a single enum variant
pub enum Variant<'a> {
    Unit(UnitVariant<'a>),
    Struct(StructVariant<'a>),
    Tuple(TupleVariant<'a>),
}

impl<'a> Variant<'a> {
    pub fn new(
        variant: &'a args::EnumVariant,
        serde_ast: &'a ast::Container,
        serde_variant: &'a serde_derive_internals::ast::Variant,
    ) -> Self {
        let ident = &variant.ident;
        let fields = field::VariantFields::new(variant, serde_variant);
        let name = serde_variant.attrs.name().deserialize_name();
        let tag = serde_ast.attrs.tag();
        let span = ident.span();

        match variant.fields.style {
            Style::Unit => Self::Unit(UnitVariant { span, tag, name }),
            Style::Tuple => Self::Tuple(TupleVariant {
                span,
                tag,
                name,
                fields,
            }),
            Style::Struct => Self::Struct(StructVariant {
                ident,
                fields,
                serde_ast,
                attrs: &serde_variant.attrs,
            }),
        }
    }

    /// expand a single variant of an enum into a zod schema
    pub fn expand_schema(&self) -> TokenStream {
        match self {
            Variant::Unit(unit) => unit.expand_schema(),
            Variant::Struct(strukt) => strukt.expand_schema(),
            Variant::Tuple(tuple) => tuple.expand_schema(),
        }
    }

    /// expand a single enum variant to TS definition
    pub fn expand_type_def(&self) -> TokenStream {
        match self {
            Variant::Unit(unit) => unit.expand_type_defs(),
            Variant::Struct(strukt) => strukt.expand_type_defs(),
            Variant::Tuple(tuple) => tuple.expand_type_defs(),
        }
    }
}
