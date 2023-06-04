use darling::FromAttributes;
use proc_macro2::TokenStream;
use quote::ToTokens;

use super::{
    attrs::{FieldAttrsExt, NameExt, ZodFieldAttrs},
    fields::{FieldValue, ZodNamedFieldImpl, ZodUnnamedFieldImpl},
    generics,
    r#enum::{EnumImpl, TagType, VariantImpl},
    r#struct::{ZodObjectImpl, ZodTupleImpl},
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
                let inline = fields
                    .iter()
                    .filter(|field| !field.attrs.skip(derive))
                    .any(|f| generics::needs_inline(&f.ty, &input.generics));

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
                                            generics::replace_generics(&mut ty, &input.generics);
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
                                        generics::replace_generics(&mut ty, &input.generics);
                                    }

                                    if let Some(value) =
                                        ZodFieldAttrs::from_attributes(&field.original.attrs)
                                            .unwrap()
                                            .as_field_value(derive)
                                    {
                                        ZodUnnamedFieldImpl {
                                            derive,
                                            optional: field.attrs.is_optional(derive),
                                            ty: value,
                                        }
                                    } else {
                                        ZodUnnamedFieldImpl {
                                            derive,
                                            optional: field.attrs.is_optional(derive),
                                            ty: ty.into(),
                                        }
                                    }
                                })
                                .collect(),
                        },
                    ),
                    serde_derive_internals::ast::Style::Unit => todo!(),
                }
            }
            serde_derive_internals::ast::Data::Enum(variants) => {
                let inline = variants.iter().any(|v| {
                    v.fields
                        .iter()
                        .filter(|field| !field.attrs.skip(derive))
                        .any(|f| generics::needs_inline(&f.ty, &input.generics))
                });

                let variants: Vec<_> = variants
                    .into_iter()
                    .map(|variant| match input.attrs.tag().into() {
                        TagType::Externally => match variant.style {
                            serde_derive_internals::ast::Style::Unit => {
                                VariantImpl::Literal(variant.attrs.name().as_str(derive))
                            }
                            serde_derive_internals::ast::Style::Struct => {
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
                                                        generics::replace_generics(
                                                            &mut ty,
                                                            &input.generics,
                                                        );
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
                            serde_derive_internals::ast::Style::Newtype
                            | serde_derive_internals::ast::Style::Tuple => {
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
                                                        generics::replace_generics(
                                                            &mut ty,
                                                            &input.generics,
                                                        );
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
                        },
                        TagType::Internally { tag } => match variant.style {
                            serde_derive_internals::ast::Style::Unit => {
                                VariantImpl::Object(ZodObjectImpl {
                                    fields: vec![ZodNamedFieldImpl {
                                        name: tag,
                                        optional: false,
                                        derive,
                                        value: FieldValue::Literal(
                                            variant.attrs.name().as_str(derive),
                                            variant.ident.span(),
                                        ),
                                    }],
                                })
                            }
                            serde_derive_internals::ast::Style::Struct => {
                                let first = ZodNamedFieldImpl {
                                    name: tag.clone(),
                                    optional: false,
                                    derive,
                                    value: FieldValue::Literal(
                                        variant.attrs.name().as_str(derive),
                                        variant.ident.span(),
                                    ),
                                };
                                let fields = variant
                                    .fields
                                    .iter()
                                    .filter(|field| !field.attrs.skip(derive))
                                    .map(|f| {
                                        let mut ty = f.ty.clone();

                                        if !inline {
                                            generics::replace_generics(&mut ty, &input.generics)
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
                            serde_derive_internals::ast::Style::Tuple => {
                                unreachable!("prevented by serde")
                            }
                            serde_derive_internals::ast::Style::Newtype => {
                                todo!("Serde supports object merging")
                            }
                        },
                        TagType::Adjacently { tag, content } => match variant.style {
                            serde_derive_internals::ast::Style::Unit => {
                                VariantImpl::Object(ZodObjectImpl {
                                    fields: vec![ZodNamedFieldImpl {
                                        name: tag,
                                        optional: false,
                                        derive,
                                        value: FieldValue::Literal(
                                            variant.attrs.name().as_str(derive),
                                            variant.ident.span(),
                                        ),
                                    }],
                                })
                            }
                            serde_derive_internals::ast::Style::Struct => {
                                let value = FieldValue::Object(ZodObjectImpl {
                                    fields: variant
                                        .fields
                                        .iter()
                                        .filter(|field| !field.attrs.skip(derive))
                                        .map(|f| {
                                            let mut ty = f.ty.clone();
                                            if !inline {
                                                generics::replace_generics(&mut ty, &input.generics)
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
                            serde_derive_internals::ast::Style::Newtype
                            | serde_derive_internals::ast::Style::Tuple => {
                                let value = FieldValue::Tuple(ZodTupleImpl {
                                    fields: variant
                                        .fields
                                        .into_iter()
                                        .filter(|field| !field.attrs.skip(derive))
                                        .map(|f| {
                                            let mut ty = f.ty.clone();
                                            if !inline {
                                                generics::replace_generics(&mut ty, &input.generics)
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
                        },
                        TagType::Untagged => match variant.style {
                            serde_derive_internals::ast::Style::Struct => {
                                VariantImpl::Object(ZodObjectImpl {
                                    fields: variant
                                        .fields
                                        .into_iter()
                                        .filter(|field| !field.attrs.skip(derive))
                                        .map(|f| {
                                            let mut ty = f.ty.clone();
                                            if !inline {
                                                generics::replace_generics(&mut ty, input.generics)
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
                            serde_derive_internals::ast::Style::Newtype
                            | serde_derive_internals::ast::Style::Tuple => {
                                VariantImpl::Tuple(ZodTupleImpl {
                                    fields: variant
                                        .fields
                                        .into_iter()
                                        .filter(|field| !field.attrs.skip(derive))
                                        .map(|f| {
                                            let mut ty = f.ty.clone();
                                            if !inline {
                                                generics::replace_generics(&mut ty, input.generics)
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
                            serde_derive_internals::ast::Style::Unit => {
                                VariantImpl::Literal(variant.attrs.name().as_str(derive))
                            }
                        },
                    })
                    .collect();

                Self::Enum(
                    inline,
                    EnumImpl {
                        tag: input.attrs.tag().into(),
                        variants,
                        derive,
                    },
                )
            }
        }
    }

    pub(super) fn inline(&self) -> bool {
        match self {
            Data::Struct(inline, _) => *inline,
            Data::Tuple(inline, _) => *inline,
            Data::Enum(inline, _) => *inline,
        }
    }
}
