use darling::FromAttributes;
use proc_macro2::TokenStream;
use quote::ToTokens;

use super::{
    attrs::{FieldAttrsExt, NameExt, ZodFieldAttrs},
    fields::{FieldValue, ZodNamedFieldImpl, ZodUnnamedFieldImpl},
    generics,
    r#enum::{EnumImpl, TagType},
    r#struct::{ZodObjectImpl, ZodTupleImpl},
    variant::VariantImpl,
    Derive,
};

#[derive(Debug, PartialEq, Clone)]
pub(super) enum Data {
    Struct(bool, ZodObjectImpl),
    Tuple(bool, ZodTupleImpl),
    Enum(bool, EnumImpl),
}

impl ToTokens for Data {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Data::Struct(_, inner) => inner.to_tokens(tokens),
            Data::Tuple(_, inner) => inner.to_tokens(tokens),
            Data::Enum(_, inner) => inner.to_tokens(tokens),
        }
    }
}

impl Data {
    pub(super) fn new(derive: Derive, input: serde_derive_internals::ast::Container) -> Self {
        match input.data {
            serde_derive_internals::ast::Data::Struct(style, fields) => {
                Self::new_struct(derive, input.generics, style, fields)
            }
            serde_derive_internals::ast::Data::Enum(variants) => {
                Self::new_enum(derive, &input.generics, variants, input.attrs.tag().into())
            }
        }
    }

    fn new_struct(
        derive: Derive,
        generics: &syn::Generics,
        style: serde_derive_internals::ast::Style,
        fields: Vec<serde_derive_internals::ast::Field>,
    ) -> Self {
        let inline = fields
            .iter()
            .filter(|field| !field.attrs.skip(derive))
            .any(|f| generics::needs_inline(&f.ty, &generics));

        match style {
            serde_derive_internals::ast::Style::Struct => Self::Struct(
                inline,
                ZodObjectImpl {
                    fields: fields
                        .into_iter()
                        .filter(|field| !field.attrs.skip(derive))
                        .map(|field| {
                            if let Some(value) =
                                ZodFieldAttrs::from_attributes(&field.original.attrs)
                                    .unwrap()
                                    .as_field_value(derive)
                            {
                                ZodNamedFieldImpl {
                                    name: field.attrs.name().as_str(derive),
                                    optional: field.attrs.is_optional(derive),
                                    derive,
                                    value,
                                }
                            } else {
                                let mut ty = field.ty.clone();
                                if !inline {
                                    generics::replace_generics(&mut ty, &generics);
                                }

                                ZodNamedFieldImpl {
                                    name: field.attrs.name().as_str(derive),
                                    optional: field.attrs.is_optional(derive),
                                    derive,
                                    value: FieldValue::Type(ty),
                                }
                            }
                        })
                        .collect(),
                },
            ),
            serde_derive_internals::ast::Style::Newtype
            | serde_derive_internals::ast::Style::Tuple => Self::Tuple(
                inline,
                ZodTupleImpl {
                    fields: fields
                        .into_iter()
                        .filter(|field| !field.attrs.skip(derive))
                        .map(|field| {
                            let mut ty = field.ty.clone();
                            if !inline {
                                generics::replace_generics(&mut ty, &generics);
                            }

                            let value = if let Some(value) =
                                ZodFieldAttrs::from_attributes(&field.original.attrs)
                                    .unwrap()
                                    .as_field_value(derive)
                            {
                                value
                            } else {
                                ty.into()
                            };

                            ZodUnnamedFieldImpl {
                                derive,
                                optional: field.attrs.is_optional(derive),
                                ty: value,
                            }
                        })
                        .collect(),
                },
            ),
            serde_derive_internals::ast::Style::Unit => todo!(),
        }
    }

    fn new_enum(
        derive: Derive,
        generics: &syn::Generics,
        variants: Vec<serde_derive_internals::ast::Variant>,
        tag: TagType,
    ) -> Self {
        let inline = variants.iter().any(|v| {
            v.fields
                .iter()
                .filter(|field| !field.attrs.skip(derive))
                .any(|f| generics::needs_inline(&f.ty, &generics))
        });

        let variants: Vec<_> = variants
            .into_iter()
            .map(|variant| VariantImpl::new(derive, inline, &generics, variant, tag.clone()))
            .collect();

        Self::Enum(
            inline,
            EnumImpl {
                tag,
                variants,
                derive,
            },
        )
    }

    pub(super) fn inline(&self) -> bool {
        match self {
            Data::Struct(inline, _) => *inline,
            Data::Tuple(inline, _) => *inline,
            Data::Enum(inline, _) => *inline,
        }
    }
}
