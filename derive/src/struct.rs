use crate::docs::RustDocs;
use crate::field::Field;
use crate::utils::{get_zod, is_export};
use darling::ast::Fields;
use darling::ToTokens;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

pub struct Struct<'a> {
    pub(crate) ident: Ident,
    pub(crate) generics: syn::Generics,
    pub(crate) fields: Fields<Field>,
    pub(crate) docs: &'a RustDocs,
    pub(crate) ns: syn::Path,
}

enum Schema<'a> {
    Object(ObjectSchema<'a>),
    Tuple(TupleSchema<'a>),
}

struct ObjectSchema<'a> {
    fields: Vec<(String, &'a Type)>,
}

impl<'a> ToTokens for ObjectSchema<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let fields = self.fields.iter().map(|(name, ty)| {
            quote! {
                #zod::core::ast::NamedField::new::<#ty>(#name)
            }
        });
        tokens.extend(quote! {
            #zod::core::ast::ObjectSchema::new(&[#(#fields),*])
        })
    }
}

struct TupleSchema<'a> {
    fields: &'a Fields<Field>,
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
            }
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
    docs: &'a RustDocs,
    ns: &'a syn::Path,
    name: String,
    schema: Schema<'a>,
}

impl<'a> ToTokens for Export<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let docs = self.docs;
        let name = &self.name;
        let ns = &self.ns;

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
        let schema = match self.fields.style {
            darling::ast::Style::Tuple => Schema::Tuple(TupleSchema {
                fields: &self.fields,
            }),

            darling::ast::Style::Struct => Schema::Object(ObjectSchema {
                fields: self
                    .fields
                    .iter()
                    .map(|f| (f.ident.as_ref().expect("named").to_string(), &f.ty))
                    .collect(),
            }),

            darling::ast::Style::Unit => unreachable!(),
        };

        if is_export(&self.fields, &self.generics) {
            Export {
                docs: self.docs,
                ns: &self.ns,
                name: self.ident.to_string(),
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
    use crate::test_utils::compare;
    use darling::ast::Style;
    use syn::parse_quote;

    #[test]
    fn empty_named_ok() {
        let input = Struct {
            ident: parse_quote!(MyStruct),
            ns: parse_quote!(Ns),
            generics: Default::default(),
            fields: Fields::new(Style::Struct, Vec::new()),
            docs: &Default::default(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::Definition::exported(
                ::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyStruct"),
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
            ident: parse_quote!(MyStruct),
            ns: parse_quote!(Ns),
            generics: Default::default(),
            fields: Fields::new(
                Style::Struct,
                vec![
                    Field {
                        ident: parse_quote!(field1),
                        ty: parse_quote!(Vec<String>),
                        attrs: Vec::new(),
                    },
                    Field {
                        ident: parse_quote!(field2),
                        ty: parse_quote!(Option<bool>),
                        attrs: Vec::new(),
                    },
                ],
            ),
            docs: &Default::default(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::Definition::exported(
                ::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyStruct"),
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
            ident: parse_quote!(MyStruct),
            ns: parse_quote!(Ns),
            generics: Default::default(),
            fields: Fields::new(Style::Tuple, Vec::new()),
            docs: &Default::default(),
        };

        compare(
            quote!(#input),
            quote! {
                ::zod::core::ast::Definition::exported(::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyStruct"),
                    schema: ::zod::core::ast::ExportSchema::Tuple(::zod::core::ast::TupleSchema::new(&[]))
                }, &[])
            },
        )
    }

    #[test]
    fn tuple_with_fields_ok() {
        let input = Struct {
            ident: parse_quote!(MyStruct),
            ns: parse_quote!(Ns),
            generics: Default::default(),
            fields: Fields::new(
                Style::Tuple,
                vec![
                    Field {
                        ident: None,
                        ty: parse_quote!(Vec<String>),
                        attrs: Vec::new(),
                    },
                    Field {
                        ident: None,
                        ty: parse_quote!(Option<bool>),
                        attrs: Vec::new(),
                    },
                ],
            ),
            docs: &Default::default(),
        };

        compare(
            quote!(#input),
            quote! {
                ::zod::core::ast::Definition::exported(::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyStruct"),
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
            ident: parse_quote!(MyStruct),
            ns: parse_quote!(Ns),
            generics: parse_quote!(<T1, T2>),
            fields: Fields::new(
                Style::Struct,
                vec![
                    Field {
                        ident: parse_quote!(field1),
                        ty: parse_quote!(Vec<String>),
                        attrs: Vec::new(),
                    },
                    Field {
                        ident: parse_quote!(field2),
                        ty: parse_quote!(Option<bool>),
                        attrs: Vec::new(),
                    },
                    Field {
                        ident: parse_quote!(field3),
                        ty: parse_quote!(T1),
                        attrs: Vec::new(),
                    },
                    Field {
                        ident: parse_quote!(field4),
                        ty: parse_quote!(T2),
                        attrs: Vec::new(),
                    },
                ],
            ),
            docs: &Default::default(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::Definition::exported(
                ::zod::core::ast::Export {
                    docs: None,
                    path: ::zod::core::ast::Path::new::<Ns>("MyStruct"),
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
            ident: parse_quote!(MyStruct),
            ns: parse_quote!(Ns),
            generics: parse_quote!(<T1, T2>),
            fields: Fields::new(
                Style::Struct,
                vec![
                    Field {
                        ident: parse_quote!(field1),
                        ty: parse_quote!(Vec<String>),
                        attrs: Vec::new(),
                    },
                    Field {
                        ident: parse_quote!(field2),
                        ty: parse_quote!(Option<T1>),
                        attrs: Vec::new(),
                    },
                    Field {
                        ident: parse_quote!(field3),
                        ty: parse_quote!(T1),
                        attrs: Vec::new(),
                    },
                    Field {
                        ident: parse_quote!(field4),
                        ty: parse_quote!(T2),
                        attrs: Vec::new(),
                    },
                ],
            ),
            docs: &Default::default(),
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
