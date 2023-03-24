use crate::config::ContainerConfig;
use crate::field::Field;
use crate::utils::{get_zod, is_export};
use darling::ToTokens;
use proc_macro2::TokenStream;
use quote::quote;
use serde_derive_internals::ast::Style;

pub struct Struct<'a> {
    pub(crate) generics: &'a syn::Generics,
    pub(crate) fields: Vec<Field>,
    pub(crate) style: &'a Style,
    pub(crate) config: &'a ContainerConfig,
}

enum Schema<'a> {
    Object(ObjectSchema),
    Tuple(TupleSchema<'a>),
}

struct ObjectSchema {
    fields: Vec<Field>,
}

impl ToTokens for ObjectSchema {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        // let fields = self.fields.iter().map(|(name, ty)| {
        // quote! {
        // #zod::core::ast::NamedField::new::<#ty>(#name)
        // }
        // });

        let fields = &self.fields;
        tokens.extend(quote! {
            #zod::core::ast::ObjectSchema::new(&[#(#fields),*])
        })
    }
}

struct TupleSchema<'a> {
    fields: &'a [Field],
}

impl<'a> ToTokens for TupleSchema<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let fields = self.fields.iter();

        tokens.extend(quote! {
            #zod::core::ast::TupleSchema::new(&[#(#fields),*])
        })
    }
}
/// ```rust,ignore
/// ast::Definition::inlined(ast::InlineSchema::Object(ast::ObjectSchema::new(&[
///     ast::NamedField::new::<MyGeneric<String, T>>("field1"),
///     ast::NamedField::new::<MyGeneric<String, T>>("field2"),
/// ])));
/// ```
struct Inlined<'a> {
    schema: Schema<'a>,
}

impl<'a> ToTokens for Inlined<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let definition = match &self.schema {
            Schema::Object(schema) => {
                quote! {
                    #zod::core::ast::Definition::inlined(#zod::core::ast::InlineSchema::Object(#schema))
                }
            }
            Schema::Tuple(schema) => {
                quote! {
                    #zod::core::ast::Definition::inlined(#zod::core::ast::InlineSchema::Tuple(#schema))
                }
            } // Schema::Newtype(field) => {
              // let ty = &field.ty;
              // quote! {
              // #zod::core::ast::Definition::inlined(<#ty as #zod::core::Node>::inline())
              // }
              // }
        };

        tokens.extend(definition)
    }
}

/// ```rust,ignore
/// ast::Definition::exported(
///     ast::Export {
///         docs: None,
///         path: ast::Path::new::<Ns>("MyType"),
///         schema: ast::ExportSchema::Object(ast::ObjectSchema::new(&[
///             ast::NamedField::new::<Usize>("field1"),
///             ast::NamedField::new::<MyType<String>>("field2")
///         ])),
///     },
///     &[],
/// );
/// ```
struct Export<'a> {
    config: &'a ContainerConfig,
    schema: Schema<'a>,
}

impl<'a> ToTokens for Export<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let docs = &self.config.docs;
        let name = &self.config.name;
        let ns = &self.config.namespace;

        let definition = match &self.schema {
            Schema::Object(schema) => {
                quote! {
                    #zod::core::ast::Definition::exported(#zod::core::ast::Export {
                        docs: #docs,
                        path: #zod::core::ast::Path::new::<#ns>(#name),
                        schema: #zod::core::ast::ExportSchema::Object(#schema)
                    },
                    //todo
                    &[]
                    )
                }
            }
            Schema::Tuple(schema) => quote! {
                #zod::core::ast::Definition::exported(#zod::core::ast::Export {
                        docs: #docs,
                        path: #zod::core::ast::Path::new::<#ns>(#name),
                        schema: #zod::core::ast::ExportSchema::Tuple(#schema)
                },
                //todo
                &[]
                )
            },
        };

        tokens.extend(definition)
    }
}

impl<'a> ToTokens for Struct<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let schema = match self.style {
            Style::Tuple => Schema::Tuple(TupleSchema {
                fields: &self.fields,
            }),

            Style::Struct => Schema::Object(ObjectSchema {
                fields: self.fields.clone(),
            }),

            Style::Unit => unreachable!(),
            Style::Newtype => {
                let zod = get_zod();
                let ty = &self.fields.first().unwrap().ty;
                tokens.extend(
                    quote!(#zod::core::ast::Definition::inlined(<#ty as #zod::core::Node>::AST.inline())),
                );
                return;
            }
        };

        if is_export(&self.fields, &self.generics) {
            Export {
                config: &self.config,
                schema,
            }
            .to_tokens(tokens);
        } else {
            Inlined { schema }.to_tokens(tokens);
        }
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
        let input = Struct {
            config: &Default::default(),
            generics: &Default::default(),
            style: &Style::Struct,
            fields: Vec::new(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::Definition::exported(
                ::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                    schema: ::zod::core::ast::ExportSchema::Object(
                        ::zod::core::ast::ObjectSchema::new(&[])
                    )
                },
                &[]
            )),
        );
    }

    #[test]
    fn named_with_fields_ok() {
        let input = Struct {
            generics: &Default::default(),
            style: &Style::Struct,
            fields: vec![
                Field {
                    ty: parse_quote!(Vec<String>),
                    config: FieldConfig {
                        name: Some(String::from("field1")),
                        ..Default::default()
                    },
                },
                Field {
                    ty: parse_quote!(Option<bool>),
                    config: FieldConfig {
                        name: Some(String::from("field2")),
                        ..Default::default()
                    },
                },
            ],
            config: &Default::default(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::Definition::exported(
                ::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                    schema: ::zod::core::ast::ExportSchema::Object(
                        ::zod::core::ast::ObjectSchema::new(&[
                            ::zod::core::ast::NamedField::new::<Vec<String>>("field1"),
                            ::zod::core::ast::NamedField::new::<Option<bool>>("field2")
                        ])
                    )
                },
                &[]
            )),
        );
    }

    #[test]
    fn empty_tuple_ok() {
        let input = Struct {
            generics: &Default::default(),
            fields: Vec::new(),
            style: &Style::Tuple,
            config: &Default::default(),
        };

        compare(
            quote!(#input),
            quote! {
                ::zod::core::ast::Definition::exported(::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                    schema: ::zod::core::ast::ExportSchema::Tuple(::zod::core::ast::TupleSchema::new(&[]))
                }, &[])
            },
        )
    }

    #[test]
    fn tuple_with_fields_ok() {
        let input = Struct {
            generics: &Default::default(),
            style: &Style::Tuple,
            fields: vec![
                Field {
                    ty: parse_quote!(Vec<String>),
                    config: Default::default(),
                },
                Field {
                    ty: parse_quote!(Option<bool>),
                    config: Default::default(),
                },
            ],
            config: &Default::default(),
        };

        compare(
            quote!(#input),
            quote! {
                ::zod::core::ast::Definition::exported(::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                    schema: ::zod::core::ast::ExportSchema::Tuple(::zod::core::ast::TupleSchema::new(&[
                       ::zod::core::ast::TupleField::new::<Vec<String>>(),
                       ::zod::core::ast::TupleField::new::<Option<bool>>()
                    ]))
                }
                ,&[])
            },
        )
    }

    #[test]
    fn named_with_generic_fields_export_ok() {
        let input = Struct {
            generics: &parse_quote!(<T1, T2>),
            style: &Style::Struct,
            fields: vec![
                Field {
                    config: FieldConfig {
                        name: Some(String::from("field1")),
                        ..Default::default()
                    },
                    ty: parse_quote!(Vec<String>),
                },
                Field {
                    ty: parse_quote!(Option<bool>),
                    config: FieldConfig {
                        name: Some(String::from("field2")),
                        ..Default::default()
                    },
                },
                Field {
                    ty: parse_quote!(T1),
                    config: FieldConfig {
                        name: Some(String::from("field3")),
                        ..Default::default()
                    },
                },
                Field {
                    ty: parse_quote!(T2),
                    config: FieldConfig {
                        name: Some(String::from("field4")),
                        ..Default::default()
                    },
                },
            ],
            config: &Default::default(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::Definition::exported(
                ::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                    schema: ::zod::core::ast::ExportSchema::Object(
                        ::zod::core::ast::ObjectSchema::new(&[
                            ::zod::core::ast::NamedField::new::<Vec<String>>("field1"),
                            ::zod::core::ast::NamedField::new::<Option<bool>>("field2"),
                            ::zod::core::ast::NamedField::new::<T1>("field3"),
                            ::zod::core::ast::NamedField::new::<T2>("field4")
                        ])
                    )
                },
                &[]
            )),
        );
    }

    #[test]
    fn named_with_generic_fields_inline_ok() {
        let input = Struct {
            generics: &parse_quote!(<T1, T2>),
            style: &Style::Struct,
            fields: vec![
                Field {
                    ty: parse_quote!(Vec<String>),
                    config: FieldConfig {
                        name: Some(String::from("field1")),
                        ..Default::default()
                    },
                },
                Field {
                    ty: parse_quote!(Option<T1>),
                    config: FieldConfig {
                        name: Some(String::from("field2")),
                        ..Default::default()
                    },
                },
                Field {
                    ty: parse_quote!(T1),
                    config: FieldConfig {
                        name: Some(String::from("field3")),
                        ..Default::default()
                    },
                },
                Field {
                    ty: parse_quote!(T2),
                    config: FieldConfig {
                        name: Some(String::from("field4")),
                        ..Default::default()
                    },
                },
            ],
            config: &Default::default(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::Definition::inlined(
                ::zod::core::ast::InlineSchema::Object(::zod::core::ast::ObjectSchema::new(&[
                    ::zod::core::ast::NamedField::new::<Vec<String>>("field1"),
                    ::zod::core::ast::NamedField::new::<Option<T1>>("field2"),
                    ::zod::core::ast::NamedField::new::<T1>("field3"),
                    ::zod::core::ast::NamedField::new::<T2>("field4")
                ]))
            )),
        );
    }
}
