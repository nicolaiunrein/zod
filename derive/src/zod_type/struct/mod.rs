mod schema;

use crate::utils::get_zod;
use crate::zod_type::config::ContainerConfig;
use crate::zod_type::field::FilteredFields;
use darling::ToTokens;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use schema::*;
use serde_derive_internals::ast::Style;

pub(crate) struct StructExport<'a> {
    pub(crate) fields: FilteredFields,
    pub(crate) style: &'a Style,
    pub(crate) config: &'a ContainerConfig,
    pub(crate) generics: Vec<Ident>,
}

impl<'a> ToTokens for StructExport<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let docs = &self.config.docs;
        let name = &self.config.name;
        let ns = &self.config.namespace;

        let schema = match self.style {
            Style::Tuple => match TupleSchema::new(&self.fields, self.generics.clone()) {
                Ok(schema) => Schema::Tuple(schema),
                Err(err) => {
                    let err = syn::Error::from(err);
                    err.into_compile_error().to_tokens(tokens);
                    return;
                }
            },
            Style::Struct => {
                if self.config.transparent {
                    let field = self.fields.iter().next().expect("unreachable");
                    Schema::Newtype(NewtypeSchema {
                        field,
                        generics: self.generics.clone(),
                    })
                } else {
                    Schema::Object(ObjectSchema {
                        fields: self.fields.clone(),
                        generics: self.generics.clone(),
                    })
                }
            }
            Style::Unit => unreachable!(),
            Style::Newtype => {
                let field = self.fields.iter().next().expect("unreachable");

                Schema::Newtype(NewtypeSchema {
                    field,
                    generics: self.generics.clone(),
                })
            }
        };

        let export = quote! {
            #zod::core::ast::Export {
                docs: #docs,
                path: #zod::core::ast::Path::new::<#ns>(#name),
                schema: #schema,
            }
        };

        tokens.extend(export)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::compare;
    use crate::zod_type::config::FieldConfig;
    use syn::parse_quote;

    #[test]
    fn empty_named_ok() {
        let input = StructExport {
            generics: Vec::new(),
            config: &Default::default(),
            style: &Style::Struct,
            fields: FilteredFields::new(Vec::new(), Vec::new()).unwrap(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::Export {
                docs: None,
                path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                schema: ::zod::core::ast::ExportSchema::Object(
                    ::zod::core::ast::ObjectSchema::new(&[], &[]).with_extensions(&[])
                ),
            }),
        );
    }

    #[test]
    fn named_with_fields_ok() {
        let input = StructExport {
            style: &Style::Struct,
            generics: Vec::new(),

            fields: FilteredFields::new(
                vec![
                    (
                        &parse_quote!(Vec<String>),
                        FieldConfig {
                            name: Some(String::from("field1")),
                            ..Default::default()
                        },
                    ),
                    (
                        &parse_quote!(Option<bool>),
                        FieldConfig {
                            name: Some(String::from("field2")),
                            ..Default::default()
                        },
                    ),
                ],
                Vec::new(),
            )
            .unwrap(),
            config: &Default::default(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::Export {
                docs: None,
                path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                schema: ::zod::core::ast::ExportSchema::Object(
                    ::zod::core::ast::ObjectSchema::new(
                        &[
                            ::zod::core::ast::NamedField::new(
                                "field1",
                                ::zod::core::ast::Ref::new_req::<Vec<String>>()
                            ),
                            ::zod::core::ast::NamedField::new(
                                "field2",
                                ::zod::core::ast::Ref::new_req::<Option<bool>>()
                            )
                        ],
                        &[]
                    )
                    .with_extensions(&[])
                ),
            }),
        );
    }

    #[test]
    fn empty_tuple_ok() {
        let input = StructExport {
            generics: Vec::new(),
            fields: FilteredFields::new(Vec::new(), Vec::new()).unwrap(),
            style: &Style::Tuple,
            config: &Default::default(),
        };

        compare(
            quote!(#input),
            quote! {
                ::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                    schema: ::zod::core::ast::ExportSchema::Tuple(::zod::core::ast::TupleSchema::new(&[], &[])),
                }
            },
        )
    }

    #[test]
    fn tuple_with_fields_ok() {
        let input = StructExport {
            generics: Vec::new(),
            style: &Style::Tuple,
            fields: FilteredFields::new(
                vec![
                    (&parse_quote!(Vec<String>), Default::default()),
                    (&parse_quote!(Option<bool>), Default::default()),
                ],
                Vec::new(),
            )
            .unwrap(),
            config: &Default::default(),
        };

        compare(
            quote!(#input),
            quote! {
                ::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                    schema: ::zod::core::ast::ExportSchema::Tuple(::zod::core::ast::TupleSchema::new(&[
                       ::zod::core::ast::TupleField::new(::zod::core::ast::Ref::new_req::<Vec<String>>()),
                       ::zod::core::ast::TupleField::new(::zod::core::ast::Ref::new_req::<Option<bool>>())
                    ], &[])),
                }
            },
        )
    }

    #[test]
    fn named_with_generic_fields_export_ok() {
        let input = StructExport {
            generics: vec![parse_quote!(T1), parse_quote!(T2)],
            style: &Style::Struct,
            fields: FilteredFields::new(
                vec![
                    (
                        &parse_quote!(Vec<String>),
                        FieldConfig {
                            name: Some(String::from("field1")),
                            ..Default::default()
                        },
                    ),
                    (
                        &parse_quote!(Option<bool>),
                        FieldConfig {
                            name: Some(String::from("field2")),
                            ..Default::default()
                        },
                    ),
                    (
                        &parse_quote!(T1),
                        FieldConfig {
                            name: Some(String::from("field3")),
                            ..Default::default()
                        },
                    ),
                    (
                        &parse_quote!(T2),
                        FieldConfig {
                            name: Some(String::from("field4")),
                            ..Default::default()
                        },
                    ),
                ],
                vec![parse_quote!(T1), parse_quote!(T2)],
            )
            .unwrap(),
            config: &Default::default(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::Export {
                docs: None,
                path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                schema: ::zod::core::ast::ExportSchema::Object(
                    ::zod::core::ast::ObjectSchema::new(
                        &[
                            ::zod::core::ast::NamedField::new(
                                "field1",
                                ::zod::core::ast::Ref::new_req::<Vec<String>>()
                            ),
                            ::zod::core::ast::NamedField::new(
                                "field2",
                                ::zod::core::ast::Ref::new_req::<Option<bool>>()
                            ),
                            ::zod::core::ast::NamedField::new(
                                "field3",
                                ::zod::core::ast::Ref::Generic(0)
                            ),
                            ::zod::core::ast::NamedField::new(
                                "field4",
                                ::zod::core::ast::Ref::Generic(1)
                            )
                        ],
                        &["T1", "T2"]
                    )
                    .with_extensions(&[])
                ),
            }),
        );
    }

    #[test]
    fn tuple_with_generic_fields_ok() {
        let input = StructExport {
            generics: vec![parse_quote!(T1), parse_quote!(T2)],
            style: &Style::Tuple,
            fields: FilteredFields::new(
                vec![
                    (&parse_quote!(T1), Default::default()),
                    (&parse_quote!(T2), Default::default()),
                    (&parse_quote!(Option<bool>), Default::default()),
                ],
                vec![parse_quote!(T1), parse_quote!(T2)],
            )
            .unwrap(),
            config: &Default::default(),
        };

        compare(
            quote!(#input),
            quote! {
                ::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                    schema: ::zod::core::ast::ExportSchema::Tuple(::zod::core::ast::TupleSchema::new(&[
                       ::zod::core::ast::TupleField::new(::zod::core::ast::Ref::Generic(0)),
                       ::zod::core::ast::TupleField::new(::zod::core::ast::Ref::Generic(1)),
                       ::zod::core::ast::TupleField::new(::zod::core::ast::Ref::new_req::<Option<bool>>())
                    ], &["T1", "T2"])),
                }
            },
        )
    }
}
