use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use super::attrs::FieldAttrsExt;
use super::attrs::NameExt;
use super::fields::FieldValue;
use super::fields::ZodNamedFieldImpl;
use super::fields::ZodUnnamedFieldImpl;
use super::generics;
use super::r#enum::TagType;
use super::r#struct::ZodObjectImpl;
use super::Derive;
use crate::derive_internals::zod::r#struct::ZodTupleImpl;
use crate::utils::zod_core;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum VariantImpl {
    Literal(String),
    Object(ZodObjectImpl),
    Tuple(ZodTupleImpl),
}

impl VariantImpl {
    pub(super) fn new(
        derive: Derive,
        inline: bool,
        generics: &syn::Generics,
        variant: serde_derive_internals::ast::Variant,
        tag: TagType,
    ) -> Self {
        match tag {
            TagType::Externally => match variant.style {
                serde_derive_internals::ast::Style::Unit => {
                    VariantImpl::external_unit(derive, variant)
                }
                serde_derive_internals::ast::Style::Struct => {
                    VariantImpl::external_struct(derive, variant, inline, &generics)
                }
                serde_derive_internals::ast::Style::Newtype
                | serde_derive_internals::ast::Style::Tuple => {
                    VariantImpl::external_tuple(derive, variant, inline, &generics)
                }
            },
            TagType::Internally { tag } => match variant.style {
                serde_derive_internals::ast::Style::Unit => {
                    VariantImpl::internal_unit(derive, variant, tag)
                }
                serde_derive_internals::ast::Style::Struct => {
                    VariantImpl::internal_struct(derive, variant, inline, generics, tag)
                }
                serde_derive_internals::ast::Style::Tuple => {
                    unreachable!("prevented by serde")
                }
                serde_derive_internals::ast::Style::Newtype => {
                    todo!("Serde supports object merging")
                }
            },
            TagType::Adjacently { tag, content } => match variant.style {
                serde_derive_internals::ast::Style::Unit => {
                    VariantImpl::adjacent_unit(derive, variant, tag)
                }
                serde_derive_internals::ast::Style::Struct => {
                    VariantImpl::ajacent_struct(derive, variant, inline, generics, tag, content)
                }
                serde_derive_internals::ast::Style::Newtype
                | serde_derive_internals::ast::Style::Tuple => {
                    VariantImpl::ajacent_tuple(derive, variant, inline, generics, tag, content)
                }
            },
            TagType::Untagged => match variant.style {
                serde_derive_internals::ast::Style::Struct => {
                    VariantImpl::untagged_struct(derive, variant, inline, generics)
                }
                serde_derive_internals::ast::Style::Newtype
                | serde_derive_internals::ast::Style::Tuple => {
                    VariantImpl::untagged_tuple(derive, variant, inline, generics)
                }
                serde_derive_internals::ast::Style::Unit => {
                    VariantImpl::untagged_unit(derive, variant)
                }
            },
        }
    }
    fn external_unit(derive: Derive, variant: serde_derive_internals::ast::Variant) -> Self {
        VariantImpl::Literal(variant.attrs.name().as_str(derive))
    }

    fn external_struct(
        derive: Derive,
        variant: serde_derive_internals::ast::Variant,
        inline: bool,
        generics: &syn::Generics,
    ) -> Self {
        VariantImpl::Object(ZodObjectImpl {
            fields: vec![ZodNamedFieldImpl {
                name: variant.attrs.name().as_str(derive),
                optional: false,
                derive,
                value: FieldValue::Object(ZodObjectImpl {
                    fields: variant
                        .fields
                        .into_iter()
                        .filter(|field| !field.attrs.skip(derive))
                        .map(|f| {
                            let mut ty = f.ty.clone();
                            if !inline {
                                generics::replace_generics(&mut ty, &generics);
                            }

                            ZodNamedFieldImpl {
                                name: f.attrs.name().as_str(derive),
                                optional: f.attrs.is_optional(derive),
                                derive,
                                value: FieldValue::Type(ty),
                            }
                        })
                        .collect(),
                }),
            }],
        })
    }
    fn external_tuple(
        derive: Derive,
        variant: serde_derive_internals::ast::Variant,
        inline: bool,
        generics: &syn::Generics,
    ) -> Self {
        VariantImpl::Object(ZodObjectImpl {
            fields: vec![ZodNamedFieldImpl {
                name: variant.attrs.name().as_str(derive),
                optional: false,
                derive,
                value: FieldValue::Tuple(ZodTupleImpl {
                    fields: variant
                        .fields
                        .into_iter()
                        .filter(|field| !field.attrs.skip(derive))
                        .map(|f| {
                            let mut ty = f.ty.clone();
                            if !inline {
                                generics::replace_generics(&mut ty, &generics);
                            }

                            ZodUnnamedFieldImpl {
                                optional: f.attrs.is_optional(derive),
                                derive,
                                ty: ty.into(),
                            }
                        })
                        .collect(),
                }),
            }],
        })
    }

    fn internal_unit(
        derive: Derive,
        variant: serde_derive_internals::ast::Variant,
        name: String,
    ) -> Self {
        VariantImpl::Object(ZodObjectImpl {
            fields: vec![ZodNamedFieldImpl {
                name,
                optional: false,
                derive,
                value: FieldValue::Literal(
                    variant.attrs.name().as_str(derive),
                    variant.ident.span(),
                ),
            }],
        })
    }

    fn internal_struct(
        derive: Derive,
        variant: serde_derive_internals::ast::Variant,
        inline: bool,
        generics: &syn::Generics,
        name: String,
    ) -> Self {
        let first = ZodNamedFieldImpl {
            name,
            optional: false,
            derive,
            value: FieldValue::Literal(variant.attrs.name().as_str(derive), variant.ident.span()),
        };
        let fields = variant
            .fields
            .iter()
            .filter(|field| !field.attrs.skip(derive))
            .map(|f| {
                let mut ty = f.ty.clone();

                if !inline {
                    generics::replace_generics(&mut ty, &generics)
                }

                ZodNamedFieldImpl {
                    name: f.attrs.name().as_str(derive),
                    optional: f.attrs.is_optional(derive),
                    derive,
                    value: FieldValue::Type(ty),
                }
            });

        VariantImpl::Object(ZodObjectImpl {
            fields: std::iter::once(first).chain(fields).collect(),
        })
    }

    fn adjacent_unit(
        derive: Derive,
        variant: serde_derive_internals::ast::Variant,
        name: String,
    ) -> Self {
        VariantImpl::Object(ZodObjectImpl {
            fields: vec![ZodNamedFieldImpl {
                name,
                optional: false,
                derive,
                value: FieldValue::Literal(
                    variant.attrs.name().as_str(derive),
                    variant.ident.span(),
                ),
            }],
        })
    }

    fn ajacent_struct(
        derive: Derive,
        variant: serde_derive_internals::ast::Variant,
        inline: bool,
        generics: &syn::Generics,
        tag: String,
        content: String,
    ) -> Self {
        let value = FieldValue::Object(ZodObjectImpl {
            fields: variant
                .fields
                .iter()
                .filter(|field| !field.attrs.skip(derive))
                .map(|f| {
                    let mut ty = f.ty.clone();
                    if !inline {
                        generics::replace_generics(&mut ty, &generics)
                    }

                    ZodNamedFieldImpl {
                        name: f.attrs.name().as_str(derive),
                        optional: f.attrs.is_optional(derive),
                        derive,
                        value: FieldValue::Type(f.ty.clone()),
                    }
                })
                .collect(),
        });

        VariantImpl::Object(ZodObjectImpl {
            fields: vec![
                ZodNamedFieldImpl {
                    name: tag,
                    optional: false,
                    derive,
                    value: FieldValue::Literal(
                        variant.attrs.name().as_str(derive),
                        variant.ident.span(),
                    ),
                },
                ZodNamedFieldImpl {
                    name: content,
                    optional: false,
                    derive,
                    value,
                },
            ],
        })
    }

    fn ajacent_tuple(
        derive: Derive,
        variant: serde_derive_internals::ast::Variant,
        inline: bool,
        generics: &syn::Generics,
        tag: String,
        content: String,
    ) -> Self {
        let value = FieldValue::Tuple(ZodTupleImpl {
            fields: variant
                .fields
                .into_iter()
                .filter(|field| !field.attrs.skip(derive))
                .map(|f| {
                    let mut ty = f.ty.clone();
                    if !inline {
                        generics::replace_generics(&mut ty, &generics)
                    }

                    ZodUnnamedFieldImpl {
                        derive,
                        optional: f.attrs.is_optional(derive),
                        ty: ty.into(),
                    }
                })
                .collect(),
        });

        VariantImpl::Object(ZodObjectImpl {
            fields: vec![
                ZodNamedFieldImpl {
                    name: tag,
                    optional: false,
                    derive,
                    value: FieldValue::Literal(
                        variant.attrs.name().as_str(derive),
                        variant.ident.span(),
                    ),
                },
                ZodNamedFieldImpl {
                    name: content,
                    optional: false,
                    derive,
                    value,
                },
            ],
        })
    }

    fn untagged_struct(
        derive: Derive,
        variant: serde_derive_internals::ast::Variant,
        inline: bool,
        generics: &syn::Generics,
    ) -> Self {
        VariantImpl::Object(ZodObjectImpl {
            fields: variant
                .fields
                .into_iter()
                .filter(|field| !field.attrs.skip(derive))
                .map(|f| {
                    let mut ty = f.ty.clone();
                    if !inline {
                        generics::replace_generics(&mut ty, generics)
                    }
                    ZodNamedFieldImpl {
                        name: f.attrs.name().as_str(derive),
                        optional: f.attrs.is_optional(derive),
                        derive,
                        value: FieldValue::Type(ty),
                    }
                })
                .collect(),
        })
    }
    fn untagged_tuple(
        derive: Derive,
        variant: serde_derive_internals::ast::Variant,
        inline: bool,
        generics: &syn::Generics,
    ) -> Self {
        VariantImpl::Tuple(ZodTupleImpl {
            fields: variant
                .fields
                .into_iter()
                .filter(|field| !field.attrs.skip(derive))
                .map(|f| {
                    let mut ty = f.ty.clone();
                    if !inline {
                        generics::replace_generics(&mut ty, generics)
                    }
                    ZodUnnamedFieldImpl {
                        optional: f.attrs.is_optional(derive),
                        derive,
                        ty: ty.into(),
                    }
                })
                .collect(),
        })
    }
    fn untagged_unit(derive: Derive, variant: serde_derive_internals::ast::Variant) -> Self {
        VariantImpl::Literal(variant.attrs.name().as_str(derive))
    }
}

impl ToTokens for VariantImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            VariantImpl::Literal(name) => {
                quote!(#zod_core::z::ZodLiteral::String(#name).into())
            }
            VariantImpl::Object(obj) => quote!(#obj.into()),
            VariantImpl::Tuple(tuple) => quote!(#tuple.into()),
        }
        .to_tokens(tokens)
    }
}
