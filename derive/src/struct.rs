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
                #zod::core::NamedField::new<#ty>(#name)
            }
        });
        tokens.extend(quote! {
            #zod::core::ObjectSchema::new(&[#(#fields),*])
        })
    }
}

struct TupleSchema<'a> {
    fields: Vec<&'a Type>,
}

impl<'a> ToTokens for TupleSchema<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let fields = &self.fields;

        tokens.extend(quote! {
            #zod::core::TupleSchema::new(&[#(#fields),*])
        })
    }
}
/// ```rust,ignore
/// Definition::inlined(InlineSchema::Object(ObjectSchema::new(&[
///     NamedField::new::<MyGeneric<String, T>>("field1"),
///     NamedField::new::<MyGeneric<String, T>>("field2"),
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
                    #zod::core::Definition::inlined(#zod::core::InlineSchema::Object(#schema))
                }
            }
            Schema::Tuple(schema) => {
                quote! {
                    #zod::core::Definition::inlined(#zod::core::InlineSchema::Tuple(#schema))
                }
            }
        };

        tokens.extend(definition)
    }
}

/// ```rust,ignore
/// Definition::exported(
///     Export {
///         docs: None,
///         path: Path::new::<Ns>("MyType"),
///         schema: ExportSchema::Object(ObjectSchema::new(&[
///             NamedField::new::<Usize>("field1"),
///             NamedField::new::<MyType<String>>("field2")
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
                    #zod::core::Definition::exported(#zod::core::Export {
                        docs: #docs,
                        path: #zod::core::Path::new<#ns>(#name),
                        schema: #zod::core::ExportSchema::Object(#schema)
                    })
                }
            }
            Schema::Tuple(schema) => quote! {
                #zod::core::Definition::exported(#zod::core::Export {
                        docs: #docs,
                        path: #zod::core::Path::new<#ns>(#name),
                        schema: #zod::core::ExportSchema::Tuple(#schema)
                })
            },
        };

        tokens.extend(definition)
    }
}

impl<'a> ToTokens for Struct<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let schema = match self.fields.style {
            darling::ast::Style::Tuple => Schema::Tuple(TupleSchema {
                fields: self.fields.iter().map(|f| &f.ty).collect(),
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
            quote!(::zod::core::Definition::exported(::zod::core::Export {
                docs: None,
                path: ::zod::core::Path::new<Ns>("MyStruct"),
                schema: ::zod::core::ExportSchema::Object(::zod::core::ObjectSchema::new(&[]))
            })),
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
                    },
                    Field {
                        ident: parse_quote!(field2),
                        ty: parse_quote!(Option<bool>),
                    },
                ],
            ),
            docs: &Default::default(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::Definition::exported(::zod::core::Export {
                docs: None,
                path: ::zod::core::Path::new<Ns>("MyStruct"),
                schema: ::zod::core::ExportSchema::Object(::zod::core::ObjectSchema::new(&[
                     ::zod::core::NamedField::new<Vec<String>>("field1"),
                     ::zod::core::NamedField::new<Option<bool>>("field2")
                ]))
            })),
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
                ::zod::core::Definition::exported(::zod::core::Export {
                    docs: None,
                    path: ::zod::core::Path::new<Ns>("MyStruct"),
                    schema: ::zod::core::ExportSchema::Tuple(::zod::core::TupleSchema::new(&[]))
                })
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
                    },
                    Field {
                        ident: None,
                        ty: parse_quote!(Option<bool>),
                    },
                ],
            ),
            docs: &Default::default(),
        };

        compare(
            quote!(#input),
            quote! {
                ::zod::core::Definition::exported(::zod::core::Export {
                    docs: None,
                    path: ::zod::core::Path::new<Ns>("MyStruct"),
                    schema: ::zod::core::ExportSchema::Tuple(::zod::core::TupleSchema::new(&[
                       Vec<String>,
                       Option<bool>
                    ]))
                })
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
                    },
                    Field {
                        ident: parse_quote!(field2),
                        ty: parse_quote!(Option<bool>),
                    },
                    Field {
                        ident: parse_quote!(field3),
                        ty: parse_quote!(T1),
                    },
                    Field {
                        ident: parse_quote!(field4),
                        ty: parse_quote!(T2),
                    },
                ],
            ),
            docs: &Default::default(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::Definition::exported(::zod::core::Export {
                docs: None,
                path: ::zod::core::Path::new<Ns>("MyStruct"),
                schema: ::zod::core::ExportSchema::Object(::zod::core::ObjectSchema::new(&[
                     ::zod::core::NamedField::new<Vec<String>>("field1"),
                     ::zod::core::NamedField::new<Option<bool>>("field2"),
                     ::zod::core::NamedField::new<T1>("field3"),
                     ::zod::core::NamedField::new<T2>("field4")
                ]))
            })),
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
                    },
                    Field {
                        ident: parse_quote!(field2),
                        ty: parse_quote!(Option<T1>),
                    },
                    Field {
                        ident: parse_quote!(field3),
                        ty: parse_quote!(T1),
                    },
                    Field {
                        ident: parse_quote!(field4),
                        ty: parse_quote!(T2),
                    },
                ],
            ),
            docs: &Default::default(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::Definition::inlined(::zod::core::InlineSchema::Object(::zod::core::ObjectSchema::new(&[
                     ::zod::core::NamedField::new<Vec<String>>("field1"),
                     ::zod::core::NamedField::new<Option<T1>>("field2"),
                     ::zod::core::NamedField::new<T1>("field3"),
                     ::zod::core::NamedField::new<T2>("field4")
                ])))),
        );
    }
}
