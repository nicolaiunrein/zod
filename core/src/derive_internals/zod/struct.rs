use super::{fields::*, Derive};
use crate::utils::zod_core;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ZodObjectImpl {
    pub(crate) fields: Vec<ZodNamedFieldImpl>,
}

impl ToTokens for ZodObjectImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fields = &self.fields;
        tokens.extend(quote! {
            #zod_core::z::ZodObject {
                fields: ::std::vec![#(#fields),*]
            }
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct ZodTupleImpl {
    pub(crate) fields: Vec<ZodUnnamedFieldImpl>,
}

impl ZodTupleImpl {
    pub fn new(derive: Derive, fields: &syn::FieldsUnnamed) -> Self {
        Self {
            fields: fields
                .unnamed
                .iter()
                .map(|f| ZodUnnamedFieldImpl {
                    optional: false, // TODO
                    derive,
                    ty: f.ty.clone(),
                })
                .collect(),
        }
    }
}

impl ToTokens for ZodTupleImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fields = &self.fields;
        if let Some(p) = fields.iter().position(|f| f.optional) {
            if !fields.iter().skip(p).all(|f| f.optional) {
                let field = &fields[p + 1];
                syn::Error::new_spanned(
                    &field.ty,
                    "zod: `non-default field follows default field`",
                )
                .into_compile_error()
                .to_tokens(tokens);
                return;
            }
        }

        tokens.extend(quote! {
            #zod_core::z::ZodTuple {
                fields: ::std::vec![#(#fields),*]
            }
        });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::test_utils::TokenStreamExt;
    use pretty_assertions::assert_eq;
    use syn::parse_quote;

    #[test]
    fn named_struct_ok() {
        let derive = Derive::Input;
        let input = ZodObjectImpl::new(
            derive,
            &parse_quote!({
                inner_string: String,
                inner_u8: u8,
            }),
        );

        let zod_fields = vec![
            ZodNamedFieldImpl {
                name: String::from("inner_string"),
                optional: false,
                derive,
                value: FieldValue::Type(parse_quote!(String)),
            },
            ZodNamedFieldImpl {
                name: String::from("inner_u8"),
                optional: false,
                derive,
                value: FieldValue::Type(parse_quote!(u8)),
            },
        ];

        let expected = quote! {
            #zod_core::z::ZodObject {
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
        let derive = Derive::Input;
        let input = ZodTupleImpl::new(derive, &parse_quote!((String, u8)));

        let zod_fields = vec![
            ZodUnnamedFieldImpl {
                //todo
                optional: false,
                derive,
                ty: parse_quote!(String),
            },
            ZodUnnamedFieldImpl {
                optional: false,
                derive,
                ty: parse_quote!(u8),
            },
        ];

        let expected = quote! {
            #zod_core::z::ZodTuple {
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
                derive: Derive::Input,
                value: FieldValue::Type(parse_quote!(u8)),
            },
            ZodNamedFieldImpl {
                name: String::from("inner_string"),
                optional: true,
                derive: Derive::Input,
                value: FieldValue::Type(parse_quote!(String)),
            },
        ];

        let expected = quote! {
            #zod_core::z::ZodObject {
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
