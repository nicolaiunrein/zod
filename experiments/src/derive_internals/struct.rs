use super::fields::*;
use crate::utils::zod_core;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{Fields, Generics};

pub(super) struct StructImpl<Io> {
    pub(crate) ident: Ident,
    pub(crate) generics: Generics,
    pub(crate) fields: Fields,
    pub(crate) kind: Io,
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
        let kind = self.kind;
        match &self.fields {
            syn::Fields::Named(fields) => ZodObjectImpl {
                fields: fields
                    .named
                    .iter()
                    .map(|f| {
                        let name = f.ident.as_ref().expect("named field").to_string();
                        let ty = f.ty.clone();
                        ZodNamedFieldImpl {
                            name,
                            optional: false, // TODO
                            kind,
                            ty,
                        }
                    })
                    .collect(),
            }
            .to_token_stream(),
            syn::Fields::Unnamed(fields) => ZodTupleImpl {
                fields: fields
                    .unnamed
                    .iter()
                    .map(|f| ZodUnnamedFieldImpl {
                        optional: false, // TODO
                        kind,
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
        let kind = &self.kind;
        let inner = self.expand_inner();
        let arg_idents = self.args();
        let custom_suffix = self.expand_suffix();

        tokens.extend(quote!(impl #zod_core::Type<#kind> for #ident {
            type Ns = #ns;
            const NAME: &'static str = #name;

            fn value() -> #zod_core::types::ZodType<#kind> {
                #zod_core::types::ZodType {
                    optional: false,
                    custom_suffix: #custom_suffix,
                    inner: #inner.into()
                }
            }

            fn args() -> #zod_core::GenericArguments<#kind> {
                #zod_core::make_args!(#(#arg_idents),*)
            }
        }))
    }
}

struct ZodObjectImpl<Io> {
    fields: Vec<ZodNamedFieldImpl<Io>>,
}

impl<Io> ToTokens for ZodObjectImpl<Io>
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

struct ZodTupleImpl<Io> {
    fields: Vec<ZodUnnamedFieldImpl<Io>>,
}

impl<Io> ToTokens for ZodTupleImpl<Io>
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

#[cfg(test)]
mod test {
    use crate::{test_utils::TokenStreamExt, Kind};

    use super::*;
    use pretty_assertions::assert_eq;
    use syn::parse_quote;

    #[test]
    fn named_struct_ok() {
        let kind = Kind::Input;
        let input = StructImpl {
            ident: parse_quote!(Test),
            generics: parse_quote!(<T1, T2>),
            fields: syn::Fields::Named(parse_quote!({
                inner_string: String,
                inner_u8: u8,
            })),
            kind,
            ns: parse_quote!(Ns),
            custom_suffix: Some(String::from(".is_ok()")),
        };

        let zod_fields = vec![
            ZodNamedFieldImpl {
                name: String::from("inner_string"),
                optional: false,
                kind,
                ty: parse_quote!(String),
            },
            ZodNamedFieldImpl {
                name: String::from("inner_u8"),
                optional: false,
                kind,
                ty: parse_quote!(u8),
            },
        ];

        let expected = quote! {
            impl #zod_core::Type<#zod_core::Kind::Input> for Test {
                type Ns = Ns;
                const NAME: &'static str = "Test";

                fn value() -> #zod_core::types::ZodType<#kind> {
                    #zod_core::types::ZodType {
                        optional: false,
                        custom_suffix: ::std::option::Option::Some(::std::string::String::from(".is_ok()")),
                        inner: #zod_core::types::ZodObject {
                          fields: ::std::vec![ #(#zod_fields),*],
                        }.into()
                    }
                }

                fn args() -> #zod_core::GenericArguments<#kind> {
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
        let kind = Kind::Input;
        let input = StructImpl {
            ident: parse_quote!(Test),
            generics: parse_quote!(<T1, T2>),
            fields: syn::Fields::Unnamed(parse_quote!((String, u8))),
            kind,
            ns: parse_quote!(Ns),
            custom_suffix: Some(String::from(".is_ok()")),
        };

        let zod_fields = vec![
            ZodUnnamedFieldImpl {
                //todo
                optional: false,
                kind,
                ty: parse_quote!(String),
            },
            ZodUnnamedFieldImpl {
                optional: false,
                kind,
                ty: parse_quote!(u8),
            },
        ];

        let expected = quote! {
            impl #zod_core::Type<#zod_core::Kind::Input> for Test {
                type Ns = Ns;
                const NAME: &'static str = "Test";

                fn value() -> #zod_core::types::ZodType<#kind> {
                    #zod_core::types::ZodType {
                        optional: false,
                        custom_suffix: ::std::option::Option::Some(::std::string::String::from(".is_ok()")),
                        inner: #zod_core::types::ZodTuple {
                          fields: ::std::vec![ #(#zod_fields),*],
                        }.into()
                    }
                }

                fn args() -> #zod_core::GenericArguments<#kind> {
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
    fn expand_zod_object_ok() {
        let fields = ::std::vec![
            ZodNamedFieldImpl {
                name: String::from("inner_u8"),
                optional: true,
                kind: Kind::Input,
                ty: parse_quote!(u8),
            },
            ZodNamedFieldImpl {
                name: String::from("inner_string"),
                optional: true,
                kind: Kind::Input,
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

        let input = ZodObjectImpl {
            fields: fields.clone(),
        }
        .into_token_stream();

        assert_eq!(
            input.to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        )
    }
}
