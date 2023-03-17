use crate::{docs::RustDocs, get_zod};

use super::args;
use darling::ast::{Fields, Style};
use proc_macro2::TokenStream;
use quote::quote;
use serde_derive_internals::ast;
use syn::{Ident, Path};

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

    let generic_params = input
        .generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Type(t) => Some(t.ident.clone()),
            syn::GenericParam::Lifetime(_) => None,
            syn::GenericParam::Const(_) => None,
        })
        .collect::<Vec<_>>();

    let fields = fields
        .iter()
        .zip(fields_ast.iter().map(|f| &f.attrs))
        .filter(|(_, attrs)| !attrs.skip_deserializing())
        .map(|(args::StructField { ty, ident }, attrs)| StructField {
            ty,
            name: ident.as_ref().map(|_| attrs.name().deserialize_name()),
            optional: !attrs.default().is_none(),
            flatten: attrs.flatten(),
            generic_params: &generic_params,
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
        generic_params: generic_params.clone(),
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
    generics: syn::Generics,
    generic_params: Vec<Ident>,
}

impl<'a> Struct<'a> {
    fn expand(&self) -> TokenStream {
        let ident = &self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let struct_def = self.expand_struct_def();

        let zod = get_zod();

        quote! {
            const _: () = {
                const AST: #zod::ast::Item = #zod::ast::Item::Struct(#struct_def);

                impl #impl_generics #zod::ZodType for #ident #ty_generics #where_clause {
                    const AST: #zod::ast::Item = AST;
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
        let generics = self.generic_params.iter().map(|param| {
            let name = param.to_string();
            quote!(#zod::ast::Generic::Type {ident: #name} )
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
    generic_params: &'a [syn::Ident],
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
        if let Some(ident) = self.get_matching_generic() {
            let ident = ident.to_string();
            quote! {
                #zod::ast::FieldValue::Generic(#zod::ast::Generic::Type {ident: #ident })
            }
        } else {
            let ty = qualified_ty(self.ty);
            let ns_name = quote!(#ty::AST.ns());
            let name = quote!(#ty::AST.name());
            quote! {
                #zod::ast::FieldValue::Qualified(#zod::ast::QualifiedType {
                    ns: #ns_name,
                    ident: #name,
                    generics: &[] //todo
                })
            }
        }
    }

    fn get_matching_generic(&self) -> Option<Ident> {
        self.generic_params
            .iter()
            .find(|param| match self.ty {
                syn::Type::Path(p) => {
                    p.path
                        .segments
                        .iter()
                        .map(|s| s.ident.to_string())
                        .collect::<Vec<_>>()
                        == vec![param.to_string()]
                }
                _ => false,
            })
            .cloned()
    }
}
