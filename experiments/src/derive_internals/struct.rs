use crate::utils::zod_core;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{Fields, Generics};

pub(super) struct StructImpl<Io> {
    pub(crate) ident: Ident,
    pub(crate) generics: Generics,
    pub(crate) fields: Fields,
    pub(crate) role: Io,
    pub(crate) ns: syn::Path,
    pub(crate) custom_suffix: Option<String>,
}

impl<Io> StructImpl<Io>
where
    Io: ToTokens + Copy,
{
    fn args(&self) -> Vec<&Ident> {
        self.generics
            .params
            .iter()
            .map(|p| match p {
                syn::GenericParam::Lifetime(_) => todo!(),
                syn::GenericParam::Type(param) => &param.ident,
                syn::GenericParam::Const(_) => todo!(),
            })
            .collect::<Vec<_>>()
    }

    fn expand_suffix(&self) -> TokenStream2 {
        match &self.custom_suffix {
            Some(suffix) => quote!(::std::option::Option::Some(::std::string::String::from(
                #suffix
            ))),
            None => quote!(None),
        }
    }

    fn expand_inner(&self) -> TokenStream2 {
        let role = self.role;
        match &self.fields {
            syn::Fields::Named(fields) => ZodObject {
                fields: fields
                    .named
                    .iter()
                    .map(|f| {
                        let name = f.ident.as_ref().expect("named field").to_string();
                        let ty = f.ty.clone();
                        ZodNamedField {
                            name,
                            optional: false, // TODO
                            role,
                            ty,
                        }
                    })
                    .collect(),
            }
            .to_token_stream(),
            syn::Fields::Unnamed(fields) => ZodTuple {
                fields: fields
                    .unnamed
                    .iter()
                    .map(|f| ZodUnnamedField {
                        optional: false, // TODO
                        role,
                        ty: f.ty.clone(),
                    })
                    .collect(),
            }
            .to_token_stream(),
            syn::Fields::Unit => todo!(),
        }
    }
}

impl<Io> ToTokens for StructImpl<Io>
where
    Io: ToTokens + Copy,
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ns = &self.ns;
        let ident = &self.ident;
        let name = self.ident.to_string();
        let role = &self.role;
        let inner = self.expand_inner();
        let arg_idents = self.args();
        let custom_suffix = self.expand_suffix();

        tokens.extend(quote!(impl #zod_core::Type<#role> for #ident {
            type Ns = #ns;
            const NAME: &'static str = #name;

            fn value() -> #zod_core::types::ZodType<#role> {
                #zod_core::types::ZodType {
                    optional: false,
                    custom_suffix: #custom_suffix,
                    inner: #inner.into()
                }
            }

            fn args() -> #zod_core::GenericArguments<#role> {
                #zod_core::make_args!(#(#arg_idents),*)
            }
        }))
    }
}

struct ZodObject<Io> {
    fields: Vec<ZodNamedField<Io>>,
}

#[derive(Clone, Debug, PartialEq)]
struct ZodNamedField<Io> {
    name: String,
    optional: bool,
    role: Io,
    ty: syn::Type,
}

#[derive(Clone, Debug, PartialEq)]
struct ZodUnnamedField<Io> {
    role: Io,
    optional: bool,
    ty: syn::Type,
}

struct ZodTuple<Io> {
    fields: Vec<ZodUnnamedField<Io>>,
}

impl<Io> ToTokens for ZodTuple<Io>
where
    Io: ToTokens + Copy,
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let fields = &self.fields;
        tokens.extend(quote! {
            #zod_core::types::ZodTuple {
                fields: ::std::vec![#(#fields),*]
            }
        });
    }
}

impl<Io> ToTokens for ZodUnnamedField<Io>
where
    Io: ToTokens + Copy,
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let optional = &self.optional;
        let ty = &self.ty;
        let role = self.role;
        let qualified_ty = quote_spanned!(ty.span() => <#ty as #zod_core::Type::<#role>>);

        tokens.extend(quote_spanned! {
            ty.span() =>
            #zod_core::types::ZodType {
                optional: #optional,
                ..#qualified_ty::get_ref().into()
            }
        })
    }
}

impl<Io> ToTokens for ZodNamedField<Io>
where
    Io: ToTokens + Copy,
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let optional = &self.optional;
        let role = self.role;
        let ty = &self.ty;
        let qualified_ty = quote_spanned!(ty.span() => <#ty as #zod_core::Type::<#role>>);

        tokens.extend(quote_spanned! {
            ty.span() =>
            #zod_core::types::ZodNamedField {
                name: #name,
                optional: #optional,
                value: #qualified_ty::get_ref().into(),
            }
        })
    }
}

impl<Io> ToTokens for ZodObject<Io>
where
    Io: ToTokens + Copy,
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let fields = &self.fields;
        tokens.extend(quote! {
            #zod_core::types::ZodObject {
                fields: ::std::vec![#(#fields),*]
            }
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{test_utils::TokenStreamExt, Kind};

    use super::*;
    use pretty_assertions::assert_eq;
    use syn::parse_quote;

    #[test]
    fn named_struct_ok() {
        let role = Kind::Input;
        let input = StructImpl {
            ident: parse_quote!(Test),
            generics: parse_quote!(<T1, T2>),
            fields: syn::Fields::Named(parse_quote!({
                inner_string: String,
                inner_u8: u8,
            })),
            role,
            ns: parse_quote!(Ns),
            custom_suffix: Some(String::from(".is_ok()")),
        };

        let zod_fields = vec![
            ZodNamedField {
                name: String::from("inner_string"),
                optional: false,
                role,
                ty: parse_quote!(String),
            },
            ZodNamedField {
                name: String::from("inner_u8"),
                optional: false,
                role,
                ty: parse_quote!(u8),
            },
        ];

        let expected = quote! {
            impl #zod_core::Type<#zod_core::Kind::Input> for Test {
                type Ns = Ns;
                const NAME: &'static str = "Test";

                fn value() -> #zod_core::types::ZodType<#role> {
                    #zod_core::types::ZodType {
                        optional: false,
                        custom_suffix: ::std::option::Option::Some(::std::string::String::from(".is_ok()")),
                        inner: #zod_core::types::ZodObject {
                          fields: ::std::vec![ #(#zod_fields),*],
                        }.into()
                    }
                }

                fn args() -> #zod_core::GenericArguments<#role> {
                    #zod_core::make_args!(T1, T2)
                }
            }
        };

        assert_eq!(
            input.to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        );
    }

    #[test]
    fn tuple_struct_ok() {
        let role = Kind::Input;
        let input = StructImpl {
            ident: parse_quote!(Test),
            generics: parse_quote!(<T1, T2>),
            fields: syn::Fields::Unnamed(parse_quote!((String, u8))),
            role,
            ns: parse_quote!(Ns),
            custom_suffix: Some(String::from(".is_ok()")),
        };

        let zod_fields = vec![
            ZodUnnamedField {
                //todo
                optional: false,
                role,
                ty: parse_quote!(String),
            },
            ZodUnnamedField {
                optional: false,
                role,
                ty: parse_quote!(u8),
            },
        ];

        let expected = quote! {
            impl #zod_core::Type<#zod_core::Kind::Input> for Test {
                type Ns = Ns;
                const NAME: &'static str = "Test";

                fn value() -> #zod_core::types::ZodType<#role> {
                    #zod_core::types::ZodType {
                        optional: false,
                        custom_suffix: ::std::option::Option::Some(::std::string::String::from(".is_ok()")),
                        inner: #zod_core::types::ZodTuple {
                          fields: ::std::vec![ #(#zod_fields),*],
                        }.into()
                    }
                }

                fn args() -> #zod_core::GenericArguments<#role> {
                    #zod_core::make_args!(T1, T2)
                }
            }
        };

        assert_eq!(
            input.to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        );
    }

    #[test]
    fn expand_named_field_ok() {
        let role = Kind::Input;
        let input = ZodNamedField {
            name: String::from("hello"),
            role,
            optional: false,
            ty: parse_quote!(String),
        }
        .into_token_stream();

        let expected = quote! {
            #zod_core::types::ZodNamedField {
                name: "hello",
                optional: false,
                value: <String as #zod_core::Type<#role>>::get_ref().into()
            }
        };

        assert_eq!(
            input.to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        );
    }

    #[test]
    fn expand_zod_object_ok() {
        let fields = ::std::vec![
            ZodNamedField {
                name: String::from("inner_u8"),
                optional: true,
                role: Kind::Input,
                ty: parse_quote!(u8),
            },
            ZodNamedField {
                name: String::from("inner_string"),
                optional: true,
                role: Kind::Input,
                ty: parse_quote!(::std::string::String),
            },
        ];

        let expected = quote! {
            #zod_core::types::ZodObject {
                fields: ::std::vec![
                    #(#fields),*
                ]
            }
        };

        let input = ZodObject {
            fields: fields.clone(),
        }
        .into_token_stream();

        assert_eq!(
            input.to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        )
    }
}
