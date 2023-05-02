mod schema;

use crate::config::ContainerConfig;
use crate::field::FilteredFields;
use crate::utils::get_zod;
use darling::ToTokens;
use proc_macro2::TokenStream;
use quote::quote;
use schema::*;
use serde_derive_internals::ast::Style;

pub(crate) struct StructExport<'a> {
    pub(crate) fields: FilteredFields,
    pub(crate) style: &'a Style,
    pub(crate) config: &'a ContainerConfig,
}

impl<'a> ToTokens for StructExport<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let docs = &self.config.docs;
        let name = &self.config.name;
        let ns = &self.config.namespace;

        let schema = match self.style {
            Style::Tuple => match TupleSchema::new(&self.fields) {
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
                    Schema::Newtype(NewtypeSchema { field })
                } else {
                    Schema::Object(ObjectSchema {
                        fields: self.fields.clone(),
                    })
                }
            }
            Style::Unit => unreachable!(),
            Style::Newtype => {
                let field = self.fields.iter().next().expect("unreachable");

                Schema::Newtype(NewtypeSchema { field })
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
    use crate::config::FieldConfig;
    use crate::test_utils::compare;
    use syn::parse_quote;

    #[test]
    fn empty_named_ok() {
        let input = StructExport {
            config: &Default::default(),
            style: &Style::Struct,
            fields: FilteredFields::new(Vec::new(), &[]).unwrap(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::Export {
                docs: None,
                path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                schema: ::zod::core::ast::ExportSchema::Object(
                    ::zod::core::ast::ObjectSchema::new(&[])
                ),
            }),
        );
    }

    #[test]
    fn named_with_fields_ok() {
        let input = StructExport {
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
                ],
                &[],
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
                    ::zod::core::ast::ObjectSchema::new(&[
                        ::zod::core::ast::NamedField::new_req::<Vec<String>>("field1"),
                        ::zod::core::ast::NamedField::new_req::<Option<bool>>("field2")
                    ])
                ),
            }),
        );
    }

    #[test]
    fn empty_tuple_ok() {
        let input = StructExport {
            fields: FilteredFields::new(Vec::new(), &[]).unwrap(),
            style: &Style::Tuple,
            config: &Default::default(),
        };

        compare(
            quote!(#input),
            quote! {
                ::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                    schema: ::zod::core::ast::ExportSchema::Tuple(::zod::core::ast::TupleSchema::new(&[])),
                }
            },
        )
    }

    #[test]
    fn tuple_with_fields_ok() {
        let input = StructExport {
            style: &Style::Tuple,
            fields: FilteredFields::new(
                vec![
                    (&parse_quote!(Vec<String>), Default::default()),
                    (&parse_quote!(Option<bool>), Default::default()),
                ],
                &[],
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
                       ::zod::core::ast::TupleField::new_req::<Vec<String>>(),
                       ::zod::core::ast::TupleField::new_req::<Option<bool>>()
                    ])),
                }
            },
        )
    }

    #[test]
    fn named_with_generic_fields_export_ok() {
        let input = StructExport {
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
                &[&parse_quote!(T1), &parse_quote!(T2)],
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
                    ::zod::core::ast::ObjectSchema::new(&[
                        ::zod::core::ast::NamedField::new_req::<Vec<String>>("field1"),
                        ::zod::core::ast::NamedField::new_req::<Option<bool>>("field2"),
                        ::zod::core::ast::NamedField::generic("field3", "T1"),
                        ::zod::core::ast::NamedField::generic("field4", "T2")
                    ])
                ),
            }),
        );
    }
}
