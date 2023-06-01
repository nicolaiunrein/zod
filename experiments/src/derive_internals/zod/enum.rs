use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::punctuated::Punctuated;

use super::fields::FieldValue;
use super::fields::ZodNamedFieldImpl;
use super::generics::needs_inline;
use super::generics::replace_generics;
use super::r#struct::StructImpl;
use super::r#struct::ZodObjectImpl;
use super::Derive;
use crate::utils::zod_core;

#[derive(Default, Clone)]
pub enum TagType {
    #[default]
    Externally,
    Internally {
        tag: String,
    },
    Adjacently {
        tag: String,
        content: String,
    },
    Untagged,
}

pub(crate) struct EnumImpl {
    pub inline: bool,
    generics: syn::Generics,
    tag: TagType,
    variants: Vec<syn::Variant>,
    derive: Derive,
}

impl EnumImpl {
    pub fn new(
        derive: Derive,
        variants: Punctuated<syn::Variant, syn::Token![,]>,
        generics: &syn::Generics,
        tag: TagType,
    ) -> Self {
        let inline = variants
            .iter()
            .any(|v| v.fields.iter().any(|f| needs_inline(&f.ty, &generics)));

        if inline {
            let variants = variants
                .into_iter()
                .map(|mut v| {
                    for f in v.fields.iter_mut() {
                        replace_generics(&mut f.ty, &generics);
                    }
                    v
                })
                .collect::<Vec<_>>();

            EnumImpl {
                generics: generics.clone(),
                tag,
                derive,
                variants,
                inline,
            }
        } else {
            let variants = variants.into_iter().collect::<Vec<_>>();
            EnumImpl {
                generics: generics.clone(),
                inline,
                tag,
                derive,
                variants,
            }
        }
    }
    
    fn variants(&self) -> Vec<TokenStream> {
        let derive = self.derive;
        self.variants
            .iter()
            .map(|orig| {
                if orig.discriminant.is_some() {
                    todo!()
                }

                let ident = &orig.ident;
                let name = ident.to_string();


                match &self.tag {
                    TagType::Externally => match orig.fields {
                        syn::Fields::Unit => {
                            quote!(#zod_core::z::ZodLiteral::String(#name).into())
                        }
                        _ => {
                            let value = StructImpl::new(derive, orig.fields.clone(), &self.generics);
                            quote! {
                                #zod_core::z::ZodObject {
                                    fields: ::std::vec![
                                        #zod_core::z::ZodNamedField {
                                            name: #name,
                                            optional: false,
                                            value: #value.into()
                                        }
                                    ],
                                }.into()
                            }
                        }
                    },
                    TagType::Internally { tag } => {
                        match &orig.fields {
                            syn::Fields::Unit => {
                                // same as Adjacently
                                quote! {
                                    #zod_core::z::ZodObject {
                                        fields: ::std::vec![
                                            #zod_core::z::ZodNamedField {
                                                name: #tag,
                                                optional: false,
                                                value: #zod_core::z::ZodLiteral::String(#name).into()
                                            },
                                        ],
                                    }.into()
                                }
                            },
                            syn::Fields::Named(fields) => {
                                let first = ZodNamedFieldImpl {
                                    name: tag.clone(),
                                    optional: false,
                                    derive,
                                    value: FieldValue::Literal(name, ident.span())
                                };
                                let fields = fields.named.iter().map(|f| ZodNamedFieldImpl {
                                        name: f.ident.as_ref().expect("Named field").to_string(),
                                        optional: false,
                                        derive,
                                        value: f.ty.clone().into()
                                    });
                                let obj = ZodObjectImpl {
                                    fields: std::iter::once(first).chain(fields).collect(),
                                };
                                quote!(#obj.into())
                            }
                            syn::Fields::Unnamed(fields) => {
                                if fields.unnamed.len() == 1 {
                                    todo!("Serde supports object merging")
                                } else {
                                    panic!("Unsupported")
                                }
                            }
                        }
                    },
                    TagType::Adjacently { tag, content } => match orig.fields {
                        syn::Fields::Unit => {
                            // same as Externally
                            quote! {
                                #zod_core::z::ZodObject {
                                    fields: ::std::vec![
                                        #zod_core::z::ZodNamedField {
                                            name: #tag,
                                            optional: false,
                                            value: #zod_core::z::ZodLiteral::String(#name).into()
                                        },
                                    ],
                                }.into()
                            }
                        }
                        _ => {
                            let value = StructImpl::new(derive, orig.fields.clone(), &self.generics);
                            quote! {
                                #zod_core::z::ZodObject {
                                    fields: ::std::vec![
                                        #zod_core::z::ZodNamedField {
                                            name: #tag,
                                            optional: false,
                                            value: #zod_core::z::ZodLiteral::String(#name).into()
                                        },
                                        #zod_core::z::ZodNamedField {
                                            name: #content,
                                            optional: false,
                                            value: #value.into()
                                        }
                                    ],
                                }.into()
                            }
                        }
                    },
                    TagType::Untagged => {
                        match orig.fields {
                            syn::Fields::Unit => {
                                quote!(#zod_core::z::ZodLiteral::String(#name).into())
                            }
                            _ => {
                                let value = StructImpl::new(derive, orig.fields.clone(), &self.generics); 
                                quote!(#value.into())
                            }
                        }
                    }
                }
            })
            .collect()
    }
}

impl ToTokens for EnumImpl {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variants = self.variants();
        let out = match &self.tag {
            TagType::Externally | TagType::Untagged => {
                quote! {
                    #zod_core::z::ZodUnion {
                        variants: ::std::vec![#(#variants),*]
                    }
                }
            }
            TagType::Internally { tag } | TagType::Adjacently { tag, .. } => {
                quote! {
                    #zod_core::z::ZodDiscriminatedUnion {
                        tag: #tag,
                        variants: ::std::vec![#(#variants),*]
                    }
                }
            }
        };

        tokens.extend(out)
    }
}
#[cfg(test)]
mod test {
    use crate::test_utils::TokenStreamExt;

    use super::*;
    use pretty_assertions::assert_eq;
    use syn::parse_quote;

    #[test]
    fn externally_tagged_ok() {
        let derive = Derive::Input;

        let tagged = |name: &'static str, inner: StructImpl| {
            quote! {
                #zod_core::z::ZodObject {
                    fields: ::std::vec![#zod_core::z::ZodNamedField {
                        name: #name,
                        optional: false,
                        value: #inner.into()
                    }],
                }.into()
            }
        };

        let variants = vec![
            quote!(#zod_core::z::ZodLiteral::String("Unit").into()),
            tagged(
                "Tuple1",
                StructImpl {
                    fields: syn::Fields::Unnamed(parse_quote!((String))),
                    derive,
                },
            ),
            tagged(
                "Tuple2",
                StructImpl {
                    fields: syn::Fields::Unnamed(parse_quote!((String, u8))),
                    derive,
                },
            ),
            tagged(
                "Struct1",
                StructImpl {
                    fields: syn::Fields::Named(parse_quote!({ inner: String })),
                    derive,
                },
            ),
            tagged(
                "Struct2",
                StructImpl {
                    fields: syn::Fields::Named(
                        parse_quote!({ inner_string: String, inner_u8: u8 }),
                    ),
                    derive,
                },
            ),
        ];

        let input = EnumImpl {
            tag: TagType::Externally,
            variants: vec![
                parse_quote!(Unit),
                parse_quote!(Tuple1(String)),
                parse_quote!(Tuple2(String, u8)),
                parse_quote!(Struct1 { inner: String }),
                parse_quote!(Struct2 {
                    inner_string: String,
                    inner_u8: u8
                }),
            ],
            derive,
        };

        let expected = quote! {
            #zod_core::z::ZodUnion {
                variants: ::std::vec![#(#variants),*]
            }
        };

        assert_eq!(
            input.to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        );
    }

    #[test]
    fn adjacently_tagged_ok() {
        let derive = Derive::Input;

        let tag_label = "my_tag";
        let content_label = "my_content";

        let tag = TagType::Adjacently {
            tag: String::from(tag_label),
            content: String::from(content_label),
        };

        let tagged = |name: &'static str, inner: StructImpl| {
            quote! {
                #zod_core::z::ZodObject {
                    fields: ::std::vec![#zod_core::z::ZodNamedField {
                        name: #tag_label,
                        optional: false,
                        value: #zod_core::z::ZodLiteral::String(#name).into()
                    },
                    #zod_core::z::ZodNamedField {
                        name: #content_label,
                        optional: false,
                        value: #inner.into()
                    }
                    ],
                }.into()
            }
        };

        let variants = vec![
            quote! {
                #zod_core::z::ZodObject {
                    fields: ::std::vec![#zod_core::z::ZodNamedField {
                        name: #tag_label,
                        optional: false,
                        value: #zod_core::z::ZodLiteral::String("Unit").into()
                    },],
                }.into()
            },
            tagged(
                "Tuple1",
                StructImpl {
                    fields: syn::Fields::Unnamed(parse_quote!((String))),
                    derive,
                },
            ),
            tagged(
                "Tuple2",
                StructImpl {
                    fields: syn::Fields::Unnamed(parse_quote!((String, u8))),
                    derive,
                },
            ),
            tagged(
                "Struct1",
                StructImpl {
                    fields: syn::Fields::Named(parse_quote!({ inner: String })),
                    derive,
                },
            ),
            tagged(
                "Struct2",
                StructImpl {
                    fields: syn::Fields::Named(
                        parse_quote!({ inner_string: String, inner_u8: u8 }),
                    ),
                    derive,
                },
            ),
        ];

        let input = EnumImpl {
            tag,
            variants: vec![
                parse_quote!(Unit),
                parse_quote!(Tuple1(String)),
                parse_quote!(Tuple2(String, u8)),
                parse_quote!(Struct1 { inner: String }),
                parse_quote!(Struct2 {
                    inner_string: String,
                    inner_u8: u8
                }),
            ],
            derive,
        };

        let expected = quote! {
            #zod_core::z::ZodDiscriminatedUnion {
                tag: "my_tag",
                variants: ::std::vec![#(#variants),*]
            }
        };

        assert_eq!(
            input.to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        );
    }

    #[test]
    fn internally_tagged_ok() {
        let derive = Derive::Input;

        let tag_label = "my_tag";

        let tag = TagType::Internally {
            tag: String::from(tag_label),
        };

        let tagged = |name: &'static str, fields: syn::FieldsNamed| {
            let fields = fields.named.iter().map(|f| {
                let name = f.ident.as_ref().unwrap().to_string();
                let value = FieldValue::from(f.ty.clone());
                ZodNamedFieldImpl {
                    name,
                    derive,
                    optional: false,
                    value,
                }
            });
            quote! {
                #zod_core::z::ZodObject {
                    fields: ::std::vec![
                        #zod_core::z::ZodNamedField {
                            name: #tag_label,
                            optional: false,
                            value: #zod_core::z::ZodLiteral::String(#name).into(),
                        },
                        #(#fields),*
                    ]
                }.into()
            }
        };

        let variants = vec![
            quote! {
                #zod_core::z::ZodObject {
                    fields: ::std::vec![#zod_core::z::ZodNamedField {
                        name: #tag_label,
                        optional: false,
                        value: #zod_core::z::ZodLiteral::String("Unit").into()
                    },],
                }.into()
            },
            tagged("Struct1", parse_quote!({ inner: String })),
            tagged(
                "Struct2",
                parse_quote!({ inner_string: String, inner_u8: u8 }),
            ),
        ];

        let input = EnumImpl {
            tag,
            variants: vec![
                parse_quote!(Unit),
                // parse_quote!(Tuple1(SomeObject)), // TODO make possible for inner objects
                parse_quote!(Struct1 { inner: String }),
                parse_quote!(Struct2 {
                    inner_string: String,
                    inner_u8: u8
                }),
            ],
            derive,
        };

        let expected = quote! {
            #zod_core::z::ZodDiscriminatedUnion {
                tag: "my_tag",
                variants: ::std::vec![#(#variants),*],
            }
        };

        assert_eq!(
            input.to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        );
    }
}
