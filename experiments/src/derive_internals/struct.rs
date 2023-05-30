use super::fields::*;
use crate::utils::zod_core;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::Fields;

pub(super) struct StructImpl<Io> {
    pub(crate) fields: Fields,
    pub(crate) kind: Io,
}

impl<Io> ToTokens for StructImpl<Io>
where
    Io: ToTokens + Copy,
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let kind = self.kind;
        let inner = match &self.fields {
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
                            value: ty.into(),
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
        };

        tokens.extend(inner)
    }
}

pub(crate) struct ZodObjectImpl<Io> {
    pub(crate) fields: Vec<ZodNamedFieldImpl<Io>>,
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
            fields: syn::Fields::Named(parse_quote!({
                inner_string: String,
                inner_u8: u8,
            })),
            kind,
        };

        let zod_fields = vec![
            ZodNamedFieldImpl {
                name: String::from("inner_string"),
                optional: false,
                kind,
                value: FieldValue::Type(parse_quote!(String)),
            },
            ZodNamedFieldImpl {
                name: String::from("inner_u8"),
                optional: false,
                kind,
                value: FieldValue::Type(parse_quote!(u8)),
            },
        ];

        let expected = quote! {
            #zod_core::types::ZodObject {
              fields: ::std::vec![ #(#zod_fields),*],
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
            fields: syn::Fields::Unnamed(parse_quote!((String, u8))),
            kind,
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
            #zod_core::types::ZodTuple {
              fields: ::std::vec![ #(#zod_fields),*],
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
                value: FieldValue::Type(parse_quote!(u8)),
            },
            ZodNamedFieldImpl {
                name: String::from("inner_string"),
                optional: true,
                kind: Kind::Input,
                value: FieldValue::Type(parse_quote!(String)),
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
