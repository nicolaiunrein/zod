use crate::{args::InputGeneric, docs::RustDocs, get_zod};

use super::args;
use darling::ast::{Fields, Style};
use proc_macro2::TokenStream;
use quote::quote;
use serde_derive_internals::ast;
use syn::{parse_quote, Ident, Path};

fn qualified_ty(ty: &syn::Type) -> proc_macro2::TokenStream {
    let zod = get_zod();
    quote!(<#ty as #zod::ZodType>)
}

pub fn expand(
    input: args::Input,
    fields: Fields<args::StructField>,
    serde_ast: ast::Container,
    docs: RustDocs,
) -> proc_macro2::TokenStream {
    let transparent = serde_ast.attrs.transparent();

    let ident = input.ident;
    let name = serde_ast.attrs.name().deserialize_name();
    let ns_path = input.namespace;

    let fields_ast = match serde_ast.data {
        ast::Data::Enum(_) => unreachable!(),
        ast::Data::Struct(_, fields) => fields,
    };

    let style = fields.style;

    let fields = fields
        .iter()
        .zip(fields_ast.iter().map(|f| &f.attrs))
        .filter(|(_, attrs)| !attrs.skip_deserializing())
        .map(|(args::StructField { ty, ident }, attrs)| StructField {
            ty,
            name: ident.as_ref().map(|_| attrs.name().deserialize_name()),
            optional: !attrs.default().is_none(),
            flatten: attrs.flatten(),
            generic_params: &input.generics.params,
        })
        .collect();

    let from_ty = serde_ast
        .attrs
        .type_from()
        .or_else(|| serde_ast.attrs.type_try_from())
        .cloned();

    let struct_def = Struct {
        transparent,
        ns_path,
        name,
        docs,
        ident,
        fields,
        style,
        from_ty,
        generics: input.generics.clone(),
    };

    struct_def.expand()
}

struct Struct<'a> {
    transparent: bool,
    ident: Ident,
    ns_path: Path,
    docs: RustDocs,
    name: String,
    fields: Vec<StructField<'a>>,
    style: Style,
    from_ty: Option<syn::Type>,
    generics: args::InputGenerics,
}

impl<'a> Struct<'a> {
    fn expand(&self) -> TokenStream {
        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let struct_def = self.expand_struct_def();
        let zod = get_zod();
        let docs = &self.docs;

        quote! {
            const _: () = {
                const AST: #zod::ast::ZodExport = #zod::ast::ZodExport {
                    docs: #docs,
                    def: #zod::ast::ZodDefinition::Struct(#struct_def)
                };

                impl #impl_generics #zod::ZodType for #ident #ty_generics #where_clause {
                    const AST: #zod::ast::ZodExport = AST;
                }

                #zod::__private::inventory::submit!(AST);
            };
        }
    }

    fn expand_struct_def(&self) -> TokenStream {
        let zod = get_zod();
        let ns_path = &self.ns_path;
        let name = &self.name;
        let fields = self.expand_fields();
        let generics = self.expand_generics();

        quote! {
            #zod::ast::Struct {
                ns: <#ns_path as #zod::Namespace>::NAME,
                ty: #zod::ast::Type {
                    ident: #name,
                    generics: #generics,
                },
                fields: #fields,
            }
        }
    }

    fn expand_generics(&self) -> TokenStream {
        let zod = get_zod();
        let generics = self.generics.params.iter().filter_map(|param| match param {
            args::InputGeneric::Ident(ident) => {
                let name = ident.to_string();
                Some(quote!(#zod::ast::Generic::Type {ident: #name} ))
            }
            args::InputGeneric::Lifetime => None,
        });

        quote! {
            &[#(#generics),*]
        }
    }

    fn expand_fields(&self) -> TokenStream {
        let zod = get_zod();

        match self.style {
            Style::Tuple => {
                let fields = self.fields.iter().map(|f| f.expand());
                quote! {
                    #zod::ast::StructFields::Tuple(&[#(#fields),*])
                }
            }
            Style::Struct => {
                if self.transparent {
                    let field = self.fields.first().unwrap();
                    let value = field.expand_value();
                    let optional = field.optional;
                    quote! {
                        #zod::ast::StructFields::Transparent {
                            value: #value,
                            optional: #optional,
                        }
                    }
                } else {
                    let fields = self.fields.iter().map(|f| f.expand());
                    quote! {
                        #zod::ast::StructFields::Named(&[#(#fields),*])
                    }
                }
            }
            Style::Unit => todo!(),
        }
    }
}

struct StructField<'a> {
    name: Option<String>,
    ty: &'a syn::Type,
    optional: bool,
    flatten: bool,
    generic_params: &'a [InputGeneric],
}

impl<'a> StructField<'a> {
    fn expand(&self) -> TokenStream {
        let zod = get_zod();
        let optional = self.optional;
        let value = self.expand_value();

        if self.flatten {
            quote! {
                #zod::ast::MaybeFlatField::Flat(#zod::ast::FlatField {
                    value: #value,
                })
            }
        } else {
            if let Some(name) = &self.name {
                quote! {
                    #zod::ast::MaybeFlatField::Named(#zod::ast::NamedField {
                        optional: #optional,
                        name: #name,
                        value: #value,
                    })

                }
            } else {
                quote! {
                    #zod::ast::TupleField {
                        optional: #optional,
                        value: #value,
                    }
                }
            }
        }
    }

    fn expand_value(&self) -> TokenStream {
        let zod = get_zod();
        if let Some(ident) = self.get_matching_generic(self.ty) {
            let ident = ident.to_string();
            quote! {
                #zod::ast::FieldValue::Generic(#zod::ast::Generic::Type { ident: #ident })
            }
        } else {
            let mut ty_with_erased_generics = self.ty.clone();
            match &mut ty_with_erased_generics {
                syn::Type::Path(ref mut p) => {
                    for seg in p.path.segments.iter_mut() {
                        match &mut seg.arguments {
                            syn::PathArguments::None => {}
                            syn::PathArguments::AngleBracketed(args) => {
                                for inner in args.args.iter_mut() {
                                    *inner = match inner {
                                        syn::GenericArgument::Lifetime(_) => parse_quote!('static),
                                        syn::GenericArgument::Type(_) => parse_quote!(()),
                                        syn::GenericArgument::Const(_) => todo!(),
                                        syn::GenericArgument::Binding(_) => todo!(),
                                        syn::GenericArgument::Constraint(_) => todo!(),
                                    }
                                }
                            }
                            syn::PathArguments::Parenthesized(_) => todo!(),
                        }
                    }
                }
                syn::Type::Array(_) => {
                    //
                }
                _ => todo!(),
            };

            let ty = qualified_ty(&ty_with_erased_generics);

            let ns_name = quote!(#ty::AST.ns());
            let name = quote!(#ty::AST.name());

            let generics = self
                .get_generic_args()
                .into_iter()
                .map(|ty| {
                    if let Some(ident) =self.get_matching_generic(ty) {
                        let name = ident.to_string();
                        quote!(#zod::ast::Generic::Type { ident: #name })
                    } else {
                        let tt = qualified_ty(ty);
                        quote!(#zod::ast::Generic::QualifiedType { ns: #tt::AST.ns(), ident: #tt::AST.name() })
                    }
                });

            quote! {
                #zod::ast::FieldValue::Qualified(#zod::ast::QualifiedType {
                    ns: #ns_name,
                    ident: #name,
                    generics: &[ #(#generics),* ],
                })
            }
        }
    }

    fn get_matching_generic(&self, ty: &syn::Type) -> Option<Ident> {
        self.generic_params
            .iter()
            .find_map(|param| match ty {
                syn::Type::Path(p) => match param {
                    InputGeneric::Ident(ident) => {
                        if p.path
                            .segments
                            .iter()
                            .map(|s| s.ident.to_string())
                            .collect::<Vec<_>>()
                            == vec![ident.to_string()]
                        {
                            Some(ident)
                        } else {
                            None
                        }
                    }
                    InputGeneric::Lifetime => None,
                },
                _ => None,
            })
            .cloned()
    }

    fn get_generic_args(&self) -> Vec<&syn::Type> {
        match self.ty {
            syn::Type::Array(_) => todo!("1"),
            syn::Type::BareFn(_) => todo!(),
            syn::Type::Group(_) => todo!(),
            syn::Type::ImplTrait(_) => todo!(),
            syn::Type::Infer(_) => todo!(),
            syn::Type::Macro(_) => todo!(),
            syn::Type::Never(_) => todo!(),
            syn::Type::Paren(_) => todo!(),
            syn::Type::Ptr(_) => todo!(),
            syn::Type::Reference(_) => todo!(),
            syn::Type::Slice(_) => todo!(),
            syn::Type::TraitObject(_) => todo!(),
            syn::Type::Tuple(_) => todo!(),
            syn::Type::Verbatim(_) => todo!(),
            syn::Type::Path(p) => p
                .path
                .segments
                .last()
                .map(|last| match &last.arguments {
                    syn::PathArguments::None => vec![],
                    syn::PathArguments::AngleBracketed(inner) => inner
                        .args
                        .iter()
                        .filter_map(|arg| match arg {
                            syn::GenericArgument::Type(t) => Some(t),
                            _ => None,
                        })
                        .collect(),

                    syn::PathArguments::Parenthesized(_) => todo!(),
                })
                .unwrap_or_default(),
            _ => todo!(),
        }
    }
}
