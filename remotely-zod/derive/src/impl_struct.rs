use crate::args::get_rustdoc;

use super::args;
use darling::ast::{Fields, Style};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use serde_derive_internals::attr::Container;
use syn::spanned::Spanned;

pub fn expand(
    input: args::Input,
    fields: Fields<args::StructField>,
    container: Container,
) -> proc_macro2::TokenStream {
    let ident = input.ident;
    let ident_str = ident.to_string();
    let ns_path = input.namespace.clone();

    let field_schemas = expand_schemas(&fields);
    let field_type_defs = expand_type_defs(&fields);

    let docs = match get_rustdoc(&input.attrs) {
        Ok(Some(docs)) => {
            let docs = format!(
                "/**\n{}*/\n",
                docs.lines()
                    .map(|line| format!("* {}\n", line))
                    .collect::<String>()
            );
            quote!(#docs)
        }
        Ok(None) => quote!(""),
        Err(err) => err.into_compile_error().into(),
    };

    match fields.style {
        Style::Tuple => {
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
                        format!("{}.{}", <#ns_path as remotely::__private::codegen::namespace::Namespace>::NAME, #ident_str)
                    }

                    fn docs() -> Option<&'static str> {
                        Some(#docs)
                    }
                }
            }
        }
        Style::Struct => {
            quote! {
                impl remotely_zod::Codegen for #ident {
                    fn schema() -> String {
                        let fields: Vec<String> = vec![#(#field_schemas),*];
                        format!("z.object({{{}}})", fields.join("\n"))
                    }

                    fn type_def() -> String {
                        let fields: Vec<String> = vec![#(#field_type_defs),*];
                        format!("{{{}}}", fields.join("\n"))
                    }

                    fn type_name() -> String {
                        format!("{}.{}", <#ns_path as remotely::__private::codegen::namespace::Namespace>::NAME, #ident_str)
                    }

                    fn docs() -> Option<&'static str> {
                        Some(#docs)
                    }
                }
            }
        }
        Style::Unit => unreachable!(),
    }
}

fn expand_schemas(fields: &Fields<args::StructField>) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|args::StructField { ident, ty, .. }| match ident {
            Some(ident) => {
                let field_name = ident.to_string();
                quote_spanned! {ty.span() =>  format!("{}: {},", #field_name, #ty::schema()) }
            }
            None => {
                // Newtype
                quote_spanned! { ty.span() => #ty::schema() }
            }
        })
        .collect()
}

fn expand_type_defs(fields: &Fields<args::StructField>) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|args::StructField { ident, ty, .. }| match ident {
            Some(ident) => {
                let field_name = ident.to_string();
                quote_spanned! {ty.span() =>  format!("{}: {},", #field_name, #ty::type_name()) }
            }
            None => {
                // Newtype
                quote_spanned! {ty.span() => #ty::type_name()}
            }
        })
        .collect()
}
