use super::fields::*;
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

impl ToTokens for ZodTupleImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let fields = &self.fields;
        if let Some(p) = fields.iter().position(|f| f.optional) {
            if !fields.iter().skip(p).all(|f| f.optional) {
                let field = &fields[p + 1];
                syn::Error::new(
                    field.ty.span(),
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

    use crate::{derive_internals::zod::Derive, test_utils::TokenStreamExt};
    use pretty_assertions::assert_eq;
    use syn::parse_quote;

    #[test]
    fn expand_zod_tuple_ok() {
        let derive = Derive::Input;

        let fields = vec![
            ZodUnnamedFieldImpl {
                //todo
                optional: false,
                derive,
                ty: FieldValue::Type(parse_quote!(String)),
            },
            ZodUnnamedFieldImpl {
                optional: false,
                derive,
                ty: FieldValue::Type(parse_quote!(u8)),
            },
        ];

        let input = ZodTupleImpl {
            fields: fields.clone(),
        };

        let expected = quote! {
            #zod_core::z::ZodTuple {
              fields: ::std::vec![ #(#fields),*],
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
