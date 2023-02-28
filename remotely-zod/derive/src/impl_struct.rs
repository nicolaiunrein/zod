use crate::docs::RustDocs;

use super::args;
use darling::ast::{Fields, Style};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use serde_derive_internals::ast;
use syn::spanned::Spanned;

fn qualified_ty(ty: &syn::Type) -> proc_macro2::TokenStream {
    quote!(<#ty as ::remotely_zod::Codegen>)
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
    let ns_path = input.namespace.clone();

    let fields_ast = match serde_ast.data {
        ast::Data::Enum(_) => unreachable!(),
        ast::Data::Struct(_, fields) => fields,
    };

    let field_schemas = expand_schemas(transparent, &fields, &fields_ast);
    let flattened_field_schemas =
        expand_flattened_fields_schemas(transparent, &fields, &fields_ast);

    let field_type_defs = expand_type_defs(transparent, &fields, &fields_ast);

    let flattened_field_type_defs =
        expand_flattened_field_type_defs(transparent, &fields, &fields_ast);

    match (fields.style, transparent) {
        (Style::Tuple, false) => {
            let schema = field_schemas
                .first()
                .or_else(|| flattened_field_schemas.first())
                .expect("Newtype");

            let type_def = field_type_defs
                .first()
                .or_else(|| flattened_field_type_defs.first())
                .expect("Newtype");

            quote! {
                impl remotely_zod::Codegen for #ident {
                    fn schema() -> String {
                        #schema
                    }

                    fn type_def() -> String {
                        #type_def
                    }

                    fn type_name() -> String {
                        format!("{}.{}", <#ns_path as remotely::__private::codegen::namespace::Namespace>::NAME, #name)
                    }

                    fn docs() -> Option<&'static str> {
                        Some(#docs)
                    }
                }
            }
        }
        (Style::Struct, false) => {
            quote! {
                impl remotely_zod::Codegen for #ident {
                    fn schema() -> String {
                        let fields: Vec<String> = vec![#(#field_schemas),*];
                        let extensions: Vec<String> = vec![#(#flattened_field_schemas),*];
                        format!("z.object({{{}}}){}", fields.join(",\n"), extensions.join(""))
                    }

                    fn type_def() -> String {
                        let fields: Vec<String> = vec![#(#field_type_defs),*];
                        let extensions: Vec<String> = vec![#(#flattened_field_type_defs),*];
                        format!("{{{}}}{}", fields.join(",\n"), extensions.join(""))
                    }

                    fn type_name() -> String {
                        format!("{}.{}", <#ns_path as remotely::__private::codegen::namespace::Namespace>::NAME, #name)
                    }

                    fn docs() -> Option<&'static str> {
                        Some(#docs)
                    }
                }
            }
        }

        (Style::Tuple, true) => {
            let schema = field_schemas.first().expect("Newtype");
            let type_def = field_type_defs.first().expect("Newtype");
            quote! {
                impl remotely_zod::Codegen for #ident {
                    fn schema() -> String {
                        format!("{}", #schema)
                    }

                    fn type_def() -> String {
                        format!("{}", #type_def)
                    }

                    fn type_name() -> String {
                        format!("{}.{}", <#ns_path as remotely::__private::codegen::namespace::Namespace>::NAME, #name)
                    }

                    fn docs() -> Option<&'static str> {
                        Some(#docs)
                    }
                }
            }
        }
        (Style::Struct, true) => {
            let schema = field_schemas.first().expect("At least one field");
            let type_def = field_type_defs.first().expect("At least one field");
            quote! {
                impl remotely_zod::Codegen for #ident {
                    fn schema() -> String {
                        #schema
                    }

                    fn type_def() -> String {
                        #type_def
                    }

                    fn type_name() -> String {
                        format!("{}.{}", <#ns_path as remotely::__private::codegen::namespace::Namespace>::NAME, #name)
                    }

                    fn docs() -> Option<&'static str> {
                        Some(#docs)
                    }
                }
            }
        }

        (Style::Unit, _) => unreachable!(),
    }
}

fn expand_schemas(
    is_transparent: bool,
    fields: &Fields<args::StructField>,
    fields_ast: &[ast::Field<'_>],
) -> Vec<TokenStream> {
    fields
        .iter()
        .zip(fields_ast.iter().map(|f| &f.attrs))
        .filter(|(_, attrs)| !attrs.skip_deserializing())
        .filter(|(_, attrs)| !attrs.flatten())
        .map(|(args::StructField { ty, ident }, attrs)| {
            let name = attrs.name().deserialize_name();

            let maybe_optional = if !attrs.default().is_none() {
                quote!(".optional()")
            } else {
                quote!("")
            };

            let ty = qualified_ty(ty);

            match (ident, is_transparent) {
                (Some(_), false) => {
                    quote_spanned! {ty.span() =>  format!("{}: {}{},", #name, #ty::schema(), #maybe_optional) }
                }
                (Some(_), true) => {
                    quote_spanned! {ty.span() =>  format!("{}{}", #ty::schema(), #maybe_optional) }
                }
                (None, _) => {
                    quote_spanned! { ty.span() => format!("{}{}", #ty::schema(), #maybe_optional) }
                }
            }

        })
        .collect()
}

fn expand_flattened_fields_schemas(
    is_transparent: bool,
    fields: &Fields<args::StructField>,
    fields_ast: &[ast::Field<'_>],
) -> Vec<TokenStream> {
    fields
        .iter()
        .zip(fields_ast.iter().map(|f| &f.attrs))
        .filter(|(_, attrs)| !attrs.skip_deserializing())
        .filter(|(_, attrs)| attrs.flatten())
        .map(|(args::StructField { ty, ident }, attrs)| {
            let maybe_optional = if !attrs.default().is_none() {
                quote!(".optional()")
            } else {
                quote!("")
            };

            let ty = qualified_ty(ty);

            match (ident, is_transparent) {
                (Some(_), false) => {
                    quote_spanned! {ty.span() =>  format!(".extend({}{})", #ty::schema(), #maybe_optional) }
                }
                (Some(_), true) => {
                    quote_spanned! {ty.span() =>  format!(".extend({}{})", #ty::schema(), #maybe_optional) }
                }
                (None, _) => {
                    // Newtype
                    quote_spanned! { ty.span() => format!(".extend({}{})", #ty::schema(), #maybe_optional) }
                }

            }
        })
        .collect()
}

fn expand_type_defs(
    is_transparent: bool,
    fields: &Fields<args::StructField>,
    fields_ast: &[ast::Field<'_>],
) -> Vec<TokenStream> {
    fields
        .iter()
        .zip(fields_ast.iter().map(|f| &f.attrs))
        .filter(|(_, attrs)| !attrs.skip_deserializing())
        .filter(|(_, attrs)| !attrs.flatten())
        .map(|(args::StructField { ident, ty, .. }, attrs)| {
            let name = attrs.name().deserialize_name();
            let is_optional = !attrs.default().is_none();
            let ty = qualified_ty(ty);

            match (ident, is_optional, is_transparent) {
                (Some(_), false, false) => {
                    quote_spanned! {ty.span() =>  format!("{}: {}", #name, #ty::type_name()) }
                }
                (None, false, false) => {
                    // Newtype
                    quote_spanned! {ty.span() => #ty::type_name()}
                }
                (Some(_), true, false) => {
                    quote_spanned! {ty.span() =>  format!("{}?: {} | undefined", #name, #ty::type_name()) }
                }
                (None, true, false) => {
                    // Newtype
                    quote_spanned! {ty.span() => format!("{} | undefined", #ty::type_name())}
                }


                (_, false, true) => {
                    // Newtype
                    quote_spanned! {ty.span() => #ty::type_name()}
                }
                (_, true, true) => {
                    quote_spanned! {ty.span() =>  format!("{} | undefined", #ty::type_name()) }
                }
            }
        })
        .collect()
}

fn expand_flattened_field_type_defs(
    is_transparent: bool,
    fields: &Fields<args::StructField>,
    fields_ast: &[ast::Field<'_>],
) -> Vec<TokenStream> {
    fields
        .iter()
        .zip(fields_ast.iter().map(|f| &f.attrs))
        .filter(|(_, attrs)| !attrs.skip_deserializing())
        .filter(|(_, attrs)| attrs.flatten())
        .map(|(args::StructField { ident, ty, .. }, attrs)| {
            let is_optional = !attrs.default().is_none();

            let ty = qualified_ty(ty);

            match (ident, is_optional, is_transparent) {
                (Some(_), false, false) => {
                    quote_spanned! {ty.span() =>  format!(" & {}", #ty::type_name()) }
                }
                (None, false, false) => {
                    // Newtype
                    quote_spanned! {ty.span() => #ty::type_name()}
                }
                (Some(_), true, false) => {
                    quote_spanned! {ty.span() =>  format!("& ({} | undefined)", #ty::type_name()) }
                }
                (None, true, false) => {
                    // Newtype
                    quote_spanned! {ty.span() => format!("& ({} | undefined)", #ty::type_name())}
                }

                (_, false, true) => {
                    // Newtype
                    quote_spanned! {ty.span() => #ty::type_name()}
                }
                (_, true, true) => {
                    quote_spanned! {ty.span() =>  format!("& ({} | undefined)", #ty::type_name()) }
                }
            }
        })
        .collect()
}
