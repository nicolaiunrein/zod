use super::attrs::{FieldAttrsExt, NameExt};
use super::custom_suffix::CustomSuffix;
use super::fields::{FieldValue, ZodNamedFieldImpl, ZodUnnamedFieldImpl};
use super::generics::{needs_inline, replace_generics, GenericsExt};
use super::r#enum::{EnumImpl, TagType, VariantImpl};
use super::r#struct::{ZodObjectImpl, ZodTupleImpl};
use super::Derive;
use super::{attrs::ZodAttrs, custom_suffix};
use crate::utils::zod_core;
use crate::Kind;
use darling::FromDeriveInput;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::DeriveInput;

pub(super) struct Ast {
    derive: Derive,
    pub ident: syn::Ident,
    pub data: Data,
    pub generics: syn::Generics,
    pub namespace: syn::Path,
    pub custom_suffix: custom_suffix::CustomSuffix,
    pub name: String,
    pub optional: bool,
    // pub transparent: bool,
    // pub type_from: Option<syn::Type>,
    // pub type_try_from: Option<syn::Type>,
    // pub type_into: Option<syn::Type>,
}

impl Ast {
    pub fn new(derive: Derive, mut derive_input: DeriveInput) -> Result<Self, TokenStream2> {
        let cx = serde_derive_internals::Ctxt::new();
        let input_clone = derive_input.clone();
        let serde_ast =
            serde_derive_internals::ast::Container::from_ast(&cx, &input_clone, derive.into())
                .unwrap();

        cx.check().unwrap();

        let zod_attrs: ZodAttrs = match ZodAttrs::from_derive_input(&derive_input) {
            Ok(attrs) => attrs,
            Err(err) => return Err(err.write_errors()),
        };

        derive_input.generics.update_where_clause(derive);

        let name = match derive {
            Derive::Input => serde_ast.attrs.name().deserialize_name(),
            Derive::Output => serde_ast.attrs.name().serialize_name(),
        };

        let mut custom_suffix = CustomSuffix {
            inner: zod_attrs.custom_suffix,
        };

        if serde_ast.attrs.deny_unknown_fields() {
            custom_suffix.add(".strict()");
        }

        Ok(Self {
            derive,
            ident: derive_input.ident.clone(),
            optional: !serde_ast.attrs.default().is_none(),
            data: Data::new(derive, serde_ast),
            generics: derive_input.generics,
            namespace: zod_attrs.namespace,
            custom_suffix,
            name,
        })
    }

    fn generic_arguments(&self) -> Vec<TokenStream2> {
        self.generics
            .idents()
            .iter()
            .map(|ident| {
                let name = ident.to_string();

                quote_spanned! {
                    ident.span() =>
                    #zod_core::GenericArgument::new::<#ident>(#name)
                }
            })
            .collect()
    }

    fn unique_ident(&self) -> syn::Ident {
        let name = &self.name;
        match self.derive {
            Derive::Input => {
                crate::utils::make_unique_name::<Kind::Input>(&quote::format_ident!("{name}"))
            }
            Derive::Output => {
                crate::utils::make_unique_name::<Kind::Output>(&quote::format_ident!("{name}"))
            }
        }
    }
}

impl ToTokens for Ast {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ident = &self.ident;
        let ns = &self.namespace;
        let name = &self.name;
        let custom_suffix = &self.custom_suffix;
        let inline = self.data.inline();
        let inner = &self.data;
        let generic_arguments = self.generic_arguments();
        let unique_ident = self.unique_ident();
        let derive = self.derive;
        let optional = self.optional;

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        tokens.extend(quote! {
            impl #impl_generics #zod_core::Type<#derive> for #ident #ty_generics #where_clause {
                type Ns = #ns;
                const NAME: &'static str = #name;
                const INLINE: bool = #inline;

                fn value() -> #zod_core::z::ZodType<#derive> {
                    #zod_core::z::ZodType {
                        optional: #optional,
                        custom_suffix: #custom_suffix,
                        inner: #inner.into()
                    }
                }

                fn args() -> ::std::vec::Vec<#zod_core::GenericArgument<#derive>> {
                    let mut v = ::std::vec::Vec::new();
                    #(v.push(#generic_arguments);)*
                    v
                }

                fn visit_dependencies(visitor: &mut #zod_core::DependencyVisitor<#derive>) {
                    // TODO
                }
            }

            impl #ns {
                #[allow(dead_code)]
                #[allow(non_upper_case_globals)]
                const #unique_ident: () = {};
            }
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(super) enum Data {
    Struct(bool, ZodObjectImpl),
    Tuple(bool, ZodTupleImpl),
    Enum(bool, EnumImpl),
}

impl ToTokens for Data {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Data::Struct(_, inner) => inner.to_tokens(tokens),
            Data::Tuple(_, inner) => inner.to_tokens(tokens),
            Data::Enum(_, inner) => inner.to_tokens(tokens),
        }
    }
}

impl Data {
    fn new(derive: Derive, input: serde_derive_internals::ast::Container) -> Self {
        match input.data {
            serde_derive_internals::ast::Data::Struct(style, fields) => {
                let inline = fields
                    .iter()
                    .filter(|field| !field.attrs.skip(derive))
                    .any(|f| needs_inline(&f.ty, &input.generics));

                match style {
                    serde_derive_internals::ast::Style::Struct => Self::Struct(
                        inline,
                        ZodObjectImpl {
                            fields: fields
                                .into_iter()
                                .filter(|field| !field.attrs.skip(derive))
                                .map(|field| {
                                    let mut ty = field.ty.clone();
                                    if !inline {
                                        replace_generics(&mut ty, &input.generics);
                                    }
                                    ZodNamedFieldImpl {
                                        name: field.attrs.name().as_str(derive),
                                        optional: field.attrs.is_optional(derive),
                                        derive,
                                        value: FieldValue::Type(ty),
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
                                        replace_generics(&mut ty, &input.generics);
                                    }
                                    ZodUnnamedFieldImpl {
                                        derive,
                                        optional: field.attrs.is_optional(derive),
                                        ty,
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
                        .any(|f| needs_inline(&f.ty, &input.generics))
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
                                                        replace_generics(&mut ty, &input.generics);
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
                                                        replace_generics(&mut ty, &input.generics);
                                                    }

                                                    ZodUnnamedFieldImpl {
                                                        optional: f.attrs.is_optional(derive),
                                                        derive,
                                                        ty,
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
                                            replace_generics(&mut ty, &input.generics)
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
                                                replace_generics(&mut ty, &input.generics)
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
                                                replace_generics(&mut ty, &input.generics)
                                            }

                                            ZodUnnamedFieldImpl {
                                                derive,
                                                optional: f.attrs.is_optional(derive),
                                                ty,
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
                                                replace_generics(&mut ty, input.generics)
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
                                                replace_generics(&mut ty, input.generics)
                                            }
                                            ZodUnnamedFieldImpl {
                                                optional: f.attrs.is_optional(derive),
                                                derive,
                                                ty,
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

    fn inline(&self) -> bool {
        match self {
            Data::Struct(inline, _) => *inline,
            Data::Tuple(inline, _) => *inline,
            Data::Enum(inline, _) => *inline,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::derive_internals::zod::fields::{FieldValue, ZodNamedFieldImpl};

    use super::*;
    use pretty_assertions::assert_eq;
    use syn::parse_quote;

    #[test]
    fn non_inline_generics_ok() {
        let input = parse_quote! {
            struct X<T> {
                inner: Other<T>
            }
        };

        let cx = serde_derive_internals::Ctxt::new();
        let input = serde_derive_internals::ast::Container::from_ast(
            &cx,
            &input,
            serde_derive_internals::Derive::Serialize,
        )
        .unwrap();
        cx.check().unwrap();

        let data = Data::new(Derive::Input, input);

        assert_eq!(
            data,
            Data::Struct(
                false,
                ZodObjectImpl {
                    fields: vec![ZodNamedFieldImpl {
                        name: String::from("inner"),
                        optional: false,
                        derive: Derive::Input,
                        value: FieldValue::Type(
                            parse_quote!(Other<#zod_core::typed_str::TypedStr<'T', #zod_core::typed_str::End>>),
                        )
                    }]
                }
            )
        )
    }

    #[test]
    fn inline_generics_ok() {
        let input = parse_quote! {
            struct X<T: SomeTrait> {
                inner: Other<T>
            }
        };

        let cx = serde_derive_internals::Ctxt::new();
        let input = serde_derive_internals::ast::Container::from_ast(
            &cx,
            &input,
            serde_derive_internals::Derive::Serialize,
        )
        .unwrap();
        cx.check().unwrap();

        let data = Data::new(Derive::Input, input);

        assert_eq!(
            data,
            Data::Struct(
                true,
                ZodObjectImpl {
                    fields: vec![ZodNamedFieldImpl {
                        name: String::from("inner"),
                        optional: false,
                        derive: Derive::Input,
                        value: FieldValue::Type(parse_quote!(Other<T>))
                    }]
                }
            )
        )
    }
}
