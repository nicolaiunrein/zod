use super::args;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

pub fn expand(input: args::Input, variants: Vec<args::EnumVariant>) -> proc_macro2::TokenStream {
    let ident = input.ident;
    let ident_str = ident.to_string();
    let ns_path = input.namespace.clone();

    let expanded_schemas = expand_schemas(&variants);
    let expanded_type_defs = expand_type_defs(&variants);

    match variants.len() {
        0 => todo!(),
        1 => {
            let schema = expanded_schemas.first().expect("exactly one variant");
            let type_def = expanded_type_defs.first().expect("exactly one variant");

            quote! {
                impl remotely_zod::Codegen for #ident {
                    fn schema() -> String {
                        String::from(#schema)
                    }

                    fn type_def() -> String {
                        String::from(#type_def)
                    }

                    fn type_name() -> String {
                        format!("{}.{}", <#ns_path as remotely::__private::codegen::namespace::Namespace>::NAME, #ident_str)
                    }
                }
            }
        }
        _ => {
            quote! {
                impl remotely_zod::Codegen for #ident {
                    fn schema() -> String {
                        let variants: Vec<String> = vec![#(#expanded_schemas),*];
                        format!("z.union([{}])", variants.join(", "))
                    }

                    fn type_def() -> String {
                        let type_defs: Vec<String> = vec![#(#expanded_type_defs),*];
                        type_defs.join(" | ")
                    }

                    fn type_name() -> String {
                        format!("{}.{}", <#ns_path as remotely::__private::codegen::namespace::Namespace>::NAME, #ident_str)
                    }
                }
            }
        }
    }
}

fn expand_schemas(fields: &Vec<args::EnumVariant>) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|args::EnumVariant { ident, fields, .. }| {
            let ident_str = ident.to_string();

            match fields.style {
                darling::ast::Style::Tuple => {
                    let inner = fields.iter().map(|args::EnumField {ty, ..}| {
                        quote_spanned!(ty.span() => format!("{}", <#ty as remotely_zod::Codegen>::schema()))
                    }).collect::<Vec<_>>();

                    match inner.len() {
                        0 => unreachable!(),
                        1 => {
                            let first = inner.first().unwrap();
                            quote_spanned! {ident.span() =>  format!("z.object({{{}: {}}})", #ident_str, #first) }
                        }
                        _ => 
                        {
                            let expanded_inner = quote! {
                                {
                                    let inner: Vec<String> = vec![#(#inner),*];
                                    format!("z.tuple([{}])", inner.join(", "))
                                }
                            };

                            quote_spanned! {ident.span() =>  format!("z.object({{{}: {}}})", #ident_str, #expanded_inner) }
                        }

                    }

                },
                darling::ast::Style::Unit => {
                    let ident_str = ident.to_string();
                    quote_spanned!(ident.span() => format!("z.literal(\"{}\")", #ident_str))
                }

                darling::ast::Style::Struct => todo!(),
            }


        })
        .collect()
}

fn expand_type_defs(fields: &Vec<args::EnumVariant>) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|args::EnumVariant { ident, fields, .. }| {
            let ident_str = ident.to_string();

            match fields.style {
                darling::ast::Style::Tuple => {
                    let inner = fields.iter().map(|args::EnumField {ty, ..}| {
                        quote_spanned!(ty.span() => format!("{}", <#ty as remotely_zod::Codegen>::type_def()))
                    }).collect::<Vec<_>>();

                    match inner.len() {
                        0 => unreachable!(),
                        1 => {
                            let expanded_inner = inner.first().expect("exactly one variant");
                            quote_spanned! {ident.span() =>  format!("{{ {}: {} }}", #ident_str, #expanded_inner) }
                        }
                        _ => {
                            let expanded_inner = quote! {
                                {
                                    let inner: Vec<String> = vec![#(#inner),*];
                                    format!("[{}]", inner.join(", "))
                                }
                            };
                            quote_spanned! {ident.span() =>  format!("{{ {}: {} }}", #ident_str, #expanded_inner) }
                        }
                    }

                }

                darling::ast::Style::Unit => {
                    let ident_str = ident.to_string();
                    quote_spanned!(ident.span() => format!("\"{}\"", #ident_str))
                }

                darling::ast::Style::Struct => todo!(),
            }


        })
        .collect()
}
