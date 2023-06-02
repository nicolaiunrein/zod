use super::{fields::*, Derive};
// use super::Derive;
use crate::utils::zod_core;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
// use syn::Fields;

// #[derive(Debug, PartialEq)]
// pub(super) struct StructImpl {
//     pub fields: Fields,
//     pub derive: Derive,
// }
//
// impl StructImpl {
//     pub fn new(derive: Derive, fields: syn::Fields) -> Self {
//         Self { derive, fields }
//     }
// }
//
// impl ToTokens for StructImpl {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         let derive = self.derive;
//         let inner = match &self.fields {
//             syn::Fields::Named(fields) => ZodObjectImpl {
//                 fields: fields
//                     .named
//                     .iter()
//                     .map(|f| {
//                         let name = f.ident.as_ref().expect("named field").to_string();
//                         let ty = f.ty.clone();
//                         ZodNamedFieldImpl {
//                             name,
//                             optional: false, // TODO
//                             derive,
//                             value: ty.into(),
//                         }
//                     })
//                     .collect(),
//             }
//             .to_token_stream(),
//             syn::Fields::Unnamed(fields) => ZodTupleImpl {
//                 fields: fields
//                     .unnamed
//                     .iter()
//                     .map(|f| ZodUnnamedFieldImpl {
//                         optional: false, // TODO
//                         derive,
//                         ty: f.ty.clone(),
//                     })
//                     .collect(),
//             }
//             .to_token_stream(),
//             syn::Fields::Unit => todo!(),
//         };
//
//         tokens.extend(inner)
//     }
// }

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ZodObjectImpl {
    pub(crate) fields: Vec<ZodNamedFieldImpl>,
}

impl ZodObjectImpl {
    pub fn new(derive: Derive, fields: &syn::FieldsNamed) -> Self {
        Self {
            fields: fields
                .named
                .iter()
                .map(|f| ZodNamedFieldImpl {
                    name: f.ident.as_ref().expect("a name").to_string(), // TODO
                    optional: false,                                     // TODO
                    derive,
                    value: FieldValue::Type(f.ty.clone()),
                })
                .collect(),
        }
    }
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
    fields: Vec<ZodUnnamedFieldImpl>,
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