use crate::args::EnumField;

use darling::ast::Style;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

/// represents the fields inside a variant
pub struct VariantFields<'a> {
    inner: Vec<VariantField<'a>>,
}

impl<'a> VariantFields<'a> {
    pub fn new(
        variant: &'a crate::args::EnumVariant,
        serde_variant: &'a serde_derive_internals::ast::Variant,
    ) -> Self {
        let style = variant.fields.style;

        let inner = variant
            .fields
            .iter()
            .zip(&serde_variant.fields)
            .filter_map(|(enum_field, f)| {
                if !f.attrs.skip_deserializing() {
                    Some(match style {
                        Style::Tuple => VariantField::Tuple(TupleField {
                            enum_field,
                            optional: !f.attrs.default().is_none(),
                        }),
                        Style::Struct => VariantField::Named(NamedField {
                            enum_field,
                            name: f.attrs.name().deserialize_name(),
                            optional: !f.attrs.default().is_none(),
                        }),
                        Style::Unit => unreachable!(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Self { inner }
    }
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn expand_type_defs(&self) -> Vec<TokenStream> {
        self.inner.iter().map(|f| f.expand_type_def()).collect()
    }

    pub fn expand_schema(&self) -> Vec<TokenStream> {
        self.inner.iter().map(|f| f.expand_schema()).collect()
    }
}

enum VariantField<'a> {
    Named(NamedField<'a>),
    Tuple(TupleField<'a>),
}

impl<'a> VariantField<'a> {
    fn expand_type_def(&self) -> TokenStream {
        match self {
            VariantField::Named(inner) => inner.expand_type_def(),
            VariantField::Tuple(inner) => inner.expand_type_def(),
        }
    }
    fn expand_schema(&self) -> TokenStream {
        match self {
            VariantField::Named(inner) => inner.expand_schema(),
            VariantField::Tuple(inner) => inner.expand_schema(),
        }
    }
}

struct NamedField<'a> {
    enum_field: &'a EnumField,
    name: String,
    optional: bool,
}

struct TupleField<'a> {
    enum_field: &'a EnumField,
    optional: bool,
}

impl<'a> NamedField<'a> {
    fn expand_type_def(&'a self) -> TokenStream {
        let ty = &self.enum_field.ty;
        let span = self.enum_field.ty.span();
        let name = &self.name;

        if self.optional {
            quote_spanned!(span => format!("{}?: {} | undefined", #name, <#ty as remotely_zod::Codegen>::type_def()))
        } else {
            quote_spanned!(span => format!("{}: {}", #name, <#ty as remotely_zod::Codegen>::type_def()))
        }
    }

    fn expand_schema(&self) -> TokenStream {
        let name = &self.name;
        let ty = &self.enum_field.ty;
        let maybe_optional = if self.optional {
            quote!(".optional()")
        } else {
            quote!("")
        };

        quote_spanned!(ty.span() => format!("{}: {}{}", #name, <#ty as remotely_zod::Codegen>::schema(), #maybe_optional))
    }
}

impl<'a> TupleField<'a> {
    fn expand_type_def(&'a self) -> TokenStream {
        let ty = &self.enum_field.ty;
        let span = self.enum_field.ty.span();

        if self.optional {
            quote_spanned!(span => format!("{} | undefined", <#ty as remotely_zod::Codegen>::type_def()))
        } else {
            quote_spanned!(span => format!("{}", <#ty as remotely_zod::Codegen>::type_def()))
        }
    }

    fn expand_schema(&self) -> TokenStream {
        let ty = &self.enum_field.ty;
        let maybe_optional = if self.optional {
            quote!(".optional()")
        } else {
            quote!("")
        };

        quote_spanned!(ty.span() => format!("{}{}", <#ty as remotely_zod::Codegen>::schema(), #maybe_optional))
    }
}
