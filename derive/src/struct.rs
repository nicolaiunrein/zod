use crate::config::ContainerConfig;
use crate::field::FilteredFields;
use crate::node::Derive;
use crate::utils::get_zod;
use darling::ToTokens;
use proc_macro2::TokenStream;
use quote::quote;
use serde_derive_internals::ast::Style;
use syn::Type;

pub(crate) struct Struct<'a> {
    pub(crate) fields: FilteredFields,
    pub(crate) style: &'a Style,
    pub(crate) config: &'a ContainerConfig,
    pub(crate) derive: Derive,
}

impl<'a> Struct<'a> {
    pub(crate) fn dependencies(&self) -> Vec<Type> {
        self.fields.iter().map(|f| f.ty.clone()).collect::<Vec<_>>()
    }
}

enum Schema<'a> {
    Object(ObjectSchema),
    Tuple(TupleSchema<'a>),
    Newtype(NewtypeSchema),
}

struct ObjectSchema {
    fields: FilteredFields,
}

impl ToTokens for ObjectSchema {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let fields = &self.fields;
        tokens.extend(quote!(#zod::core::ast::ObjectSchema::new(&[#fields])));
    }
}

struct NewtypeSchema {
    inner: Type,
    required: bool,
    derive: Derive,
}

impl ToTokens for NewtypeSchema {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let ty = &self.inner;
        let optional = !self.required;

        let reference = match self.derive {
            Derive::Request => quote!(&#zod::core::ast::Ref::new_req::<#ty>()),
            Derive::Response => quote!(&#zod::core::ast::Ref::new_res::<#ty>()),
        };

        tokens.extend(quote! {
            #zod::core::ast::NewtypeSchema::new(#reference, #optional)
        })
    }
}

struct TupleSchema<'a> {
    fields: &'a FilteredFields,
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

        let schema = match &self.schema {
            Schema::Object(schema) => quote!(#zod::core::ast::ExportSchema::Object(#schema)),
            Schema::Tuple(schema) => quote!( #zod::core::ast::ExportSchema::Tuple(#schema)),
            Schema::Newtype(schema) => quote!(#zod::core::ast::ExportSchema::Newtype(#schema)),
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

impl<'a> ToTokens for Struct<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let schema = match self.style {
            Style::Tuple => Schema::Tuple(TupleSchema {
                fields: &self.fields,
            }),

            Style::Struct => {
                if self.config.transparent {
                    let field = self.fields.iter().next().expect("unreachable");
                    Schema::Newtype(NewtypeSchema {
                        inner: field.ty.clone(),
                        required: field.config.required,
                        derive: self.derive,
                    })
                } else {
                    Schema::Object(ObjectSchema {
                        fields: self.fields.clone(),
                    })
                }
            }
            Style::Unit => unreachable!(),
            Style::Newtype => {
                let field = self.fields.iter().next().expect("unreachable");

                Schema::Newtype(NewtypeSchema {
                    inner: field.ty.clone(),
                    required: field.config.required,
                    derive: self.derive,
                })
            }
        };

        Export {
            config: self.config,
            schema,
        }
        .to_tokens(tokens);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::FieldConfig;
    use crate::field::Field;
    use crate::test_utils::compare;
    use syn::parse_quote;

    #[test]
    fn empty_named_ok() {
        let input = Struct {
            config: &Default::default(),
            style: &Style::Struct,
            fields: FilteredFields::new(Vec::new()),
            derive: Default::default(),
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
        let input = Struct {
            style: &Style::Struct,
            fields: FilteredFields::new(vec![
                Field {
                    ty: parse_quote!(Vec<String>),
                    config: FieldConfig {
                        name: Some(String::from("field1")),
                        ..Default::default()
                    },
                    generic: None,
                },
                Field {
                    ty: parse_quote!(Option<bool>),
                    config: FieldConfig {
                        name: Some(String::from("field2")),
                        ..Default::default()
                    },
                    generic: None,
                },
            ]),
            config: &Default::default(),
            derive: Default::default(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::Export {
                docs: None,
                path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                schema: ::zod::core::ast::ExportSchema::Object(
                    ::zod::core::ast::ObjectSchema::new(&[
                        ::zod::core::ast::NamedField::new(
                            "field1",
                            ::zod::core::ast::Ref::new_req::<Vec<String>>()
                        ),
                        ::zod::core::ast::NamedField::new(
                            "field2",
                            ::zod::core::ast::Ref::new_req::<Option<bool>>()
                        )
                    ])
                ),
            }),
        );
    }

    #[test]
    fn empty_tuple_ok() {
        let input = Struct {
            fields: FilteredFields::new(Vec::new()),
            style: &Style::Tuple,
            config: &Default::default(),
            derive: Default::default(),
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
        let input = Struct {
            style: &Style::Tuple,
            fields: FilteredFields::new(vec![
                Field {
                    ty: parse_quote!(Vec<String>),
                    config: Default::default(),
                    generic: None,
                },
                Field {
                    ty: parse_quote!(Option<bool>),
                    config: Default::default(),
                    generic: None,
                },
            ]),
            config: &Default::default(),
            derive: Default::default(),
        };

        compare(
            quote!(#input),
            quote! {
                ::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                    schema: ::zod::core::ast::ExportSchema::Tuple(::zod::core::ast::TupleSchema::new(&[
                                                                //todo
                       ::zod::core::ast::TupleField::new::<Vec<String>>(),
                       ::zod::core::ast::TupleField::new::<Option<bool>>()
                    ])),
                }
            },
        )
    }

    #[test]
    fn named_with_generic_fields_export_ok() {
        let input = Struct {
            style: &Style::Struct,
            fields: FilteredFields::new(vec![
                Field {
                    config: FieldConfig {
                        name: Some(String::from("field1")),
                        ..Default::default()
                    },
                    ty: parse_quote!(Vec<String>),
                    generic: None,
                },
                Field {
                    ty: parse_quote!(Option<bool>),
                    config: FieldConfig {
                        name: Some(String::from("field2")),
                        ..Default::default()
                    },
                    generic: None,
                },
                Field {
                    ty: parse_quote!(T1),
                    config: FieldConfig {
                        name: Some(String::from("field3")),
                        ..Default::default()
                    },
                    generic: Some(parse_quote!(T1)),
                },
                Field {
                    ty: parse_quote!(T2),
                    config: FieldConfig {
                        name: Some(String::from("field4")),
                        ..Default::default()
                    },
                    generic: Some(parse_quote!(T2)),
                },
            ]),
            config: &Default::default(),
            derive: Default::default(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::Export {
                docs: None,
                path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                schema: ::zod::core::ast::ExportSchema::Object(
                    ::zod::core::ast::ObjectSchema::new(&[
                        ::zod::core::ast::NamedField::new(
                            "field1",
                            ::zod::core::ast::Ref::new_req::<Vec<String>>()
                        ),
                        ::zod::core::ast::NamedField::new(
                            "field2",
                            ::zod::core::ast::Ref::new_req::<Option<bool>>()
                        ),
                        ::zod::core::ast::NamedField::generic("field3", "T1"),
                        ::zod::core::ast::NamedField::generic("field4", "T2")
                    ])
                ),
            }),
        );
    }

    #[test]
    fn named_with_generic_fields_inline_ok() {
        let input = Struct {
            style: &Style::Struct,
            fields: FilteredFields::new(vec![
                Field {
                    ty: parse_quote!(Vec<String>),
                    config: FieldConfig {
                        name: Some(String::from("field1")),
                        ..Default::default()
                    },
                    generic: None,
                },
                Field {
                    ty: parse_quote!(Option<T1>),
                    config: FieldConfig {
                        name: Some(String::from("field2")),
                        ..Default::default()
                    },
                    generic: None,
                },
                Field {
                    ty: parse_quote!(T1),
                    config: FieldConfig {
                        name: Some(String::from("field3")),
                        ..Default::default()
                    },
                    generic: Some(parse_quote!(T1)),
                },
                Field {
                    ty: parse_quote!(T2),
                    config: FieldConfig {
                        name: Some(String::from("field4")),
                        ..Default::default()
                    },
                    generic: Some(parse_quote!(T2)),
                },
            ]),
            config: &Default::default(),
            derive: Default::default(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::Export {
                docs: None,
                path: ::zod::core::ast::Path::new::<Ns>("MyType"),
                schema: ::zod::core::ast::ExportSchema::Object(
                    ::zod::core::ast::ObjectSchema::new(&[
                        ::zod::core::ast::NamedField::new(
                            "field1",
                            ::zod::core::ast::Ref::new_req::<Vec<String>>()
                        ),
                        ::zod::core::ast::NamedField::new(
                            "field2",
                            ::zod::core::ast::Ref::new_req::<Option<T1>>()
                        ),
                        ::zod::core::ast::NamedField::generic("field3", "T1"),
                        ::zod::core::ast::NamedField::generic("field4", "T2")
                    ])
                ),
            }),
        );
    }
}
