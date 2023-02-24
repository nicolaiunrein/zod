use crate::args::EnumField;

use super::args;
use darling::ast::Fields;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Ident};

pub fn expand(input: args::Input, variants: Vec<args::EnumVariant>) -> proc_macro2::TokenStream {
    let ident = input.ident;
    let ident_str = ident.to_string();
    let ns_path = input.namespace.clone();

    match variants.len() {
        0 => proc_macro_error::abort!(
            ident.span(),
            "deriving zod on empty enums is not supported."
        ),
        1 => {
            let variant = variants.first().expect("one variant");

            let schema = expand_variant_schema(variant);
            let type_def = expand_variant_type_def(variant);

            quote! {
                impl remotely_zod::Codegen for #ident {
                    fn schema() -> String {
                        String::from(#schema)
                    }

                    fn type_def() -> String {
                        String::from(#type_def)
                    }

                    fn type_name() -> String {
                        format!("{}.{}", <#ns_path as ::remotely::__private::codegen::namespace::Namespace>::NAME, #ident_str)
                    }
                }
            }
        }
        _ => {
            let expanded_variant_schemas = variants.iter().map(expand_variant_schema);
            let expanded_variant_type_defs = variants.iter().map(expand_variant_type_def);

            quote! {
                impl remotely_zod::Codegen for #ident {
                    fn schema() -> String {
                        let variants: std::vec::Vec<String> = vec![#(#expanded_variant_schemas),*];
                        format!("z.union([{}])", variants.join(", "))
                    }

                    fn type_def() -> String {
                        let type_defs: std::vec::Vec<String> = vec![#(#expanded_variant_type_defs),*];
                        type_defs.join(" | ")
                    }

                    fn type_name() -> String {
                        format!("{}.{}", <#ns_path as ::remotely::__private::codegen::namespace::Namespace>::NAME, #ident_str)
                    }
                }
            }
        }
    }
}

/// expand a single variant of an enum into a zod schema
fn expand_variant_schema(variant: &args::EnumVariant) -> TokenStream {
    let ident_str = variant.ident.to_string();
    match variant.fields.style {
        darling::ast::Style::Unit => expand_unit_variant_schema(&variant.ident),
        darling::ast::Style::Tuple => {
            let span = variant.ident.span();
            let fields = &variant.fields;
            let expanded_fields = expand_inner_tuple_variant_schema(fields);

            match expanded_fields.len() {
                0 => {
                    // this case is handled by darling
                    unreachable!()
                }
                1 => expand_one_tuple_field_variant_schema(expanded_fields, &ident_str, span),
                _ => expand_n_tuple_fields_variant_schema(expanded_fields, &ident_str, span),
            }
        }
        darling::ast::Style::Struct => {
            let span = variant.ident.span();
            let fields = &variant.fields;
            let expanded_fields = expand_inner_struct_variant_schema(fields);

            match expanded_fields.len() {
                0 => {
                    // this case is handled by darling
                    unreachable!()
                }
                1 => expand_one_struct_field_variant_schema(expanded_fields, &ident_str, span),
                _ => expand_n_struct_fields_variant_schema(expanded_fields, &ident_str, span),
            }
        }
    }
}

fn expand_inner_tuple_variant_schema(fields: &Fields<EnumField>) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let ty = &field.ty;
            quote_spanned!(ty.span() => format!("{}", <#ty as remotely_zod::Codegen>::schema()))
        })
        .collect()
}

fn expand_unit_variant_schema(ident: &Ident) -> TokenStream {
    let ident_str = ident.to_string();
    quote_spanned!(ident.span() => format!("z.literal(\"{}\")", #ident_str))
}

fn expand_inner_struct_variant_schema(fields: &Fields<EnumField>) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let ty = &field.ty;
            let ident_str = field.ident.as_ref().expect("named field").to_string();
            quote_spanned!(ty.span() => format!("{}: {}", #ident_str, <#ty as remotely_zod::Codegen>::schema()))
        })
    .collect()
}

/// expand an enum variant with exatly one field into a zod schema
/// Example: `A(usize)  =>  z.object({ A: z.number().int().nonnegative() })`
fn expand_one_tuple_field_variant_schema(
    inner: Vec<TokenStream>,
    ident_str: &str,
    span: Span,
) -> TokenStream {
    let first = inner.first().unwrap();
    quote_spanned! {span =>  format!("z.object({{{}: {}}})", #ident_str, #first) }
}

/// expand an enum variant with exatly one field into a zod schema
/// Example: `A{ num: usize } =>  z.object({ A: z.object({ num: z.number().int().nonnegative() }) })`
fn expand_one_struct_field_variant_schema(
    inner: Vec<TokenStream>,
    ident_str: &str,
    span: Span,
) -> TokenStream {
    let first = inner.first().unwrap();
    quote_spanned! {span =>  format!("z.object({{{}: z.object({{ {} }}) }})", #ident_str, #first) }
}

/// expand an enum tuple variant with more than one field into a zod schema
/// Example: `A(usize, String)`  ->
/// `z.object({ A: z.tuple([z.number().int().nonnegative(),  z.string()]) })`
fn expand_n_tuple_fields_variant_schema(
    inner: Vec<TokenStream>,
    ident_str: &str,
    span: Span,
) -> TokenStream {
    let expanded_inner = quote! {
        {
            let inner: std::vec::Vec<String> = vec![#(#inner),*];
            format!("z.tuple([{}])", inner.join(", "))
        }
    };

    quote_spanned! {span =>  format!("z.object({{{}: {}}})", #ident_str, #expanded_inner) }
}

/// expand an enum struct variant with more than one field into a zod schema
/// Example: `A{ num: usize, s: String}` ->
/// `z.object({ A: z.object({ num: z.number().int().nonnegative(),  s: z.string()}) })`
fn expand_n_struct_fields_variant_schema(
    inner: Vec<TokenStream>,
    ident_str: &str,
    span: Span,
) -> TokenStream {
    let expanded_inner = quote! {
        {
            let inner: std::vec::Vec<String> = vec![#(#inner),*];
            format!("z.object({{ {} }})", inner.join(", "))
        }
    };

    quote_spanned! {span =>  format!("z.object({{{}: {}}})", #ident_str, #expanded_inner) }
}

/// expand an enum variant to TS definition
fn expand_variant_type_def(args::EnumVariant { ident, fields }: &args::EnumVariant) -> TokenStream {
    let ident_str = ident.to_string();
    let span = ident.span();

    match fields.style {
        darling::ast::Style::Unit => expand_unit_variant_type_defs(span, &ident_str),
        darling::ast::Style::Tuple => {
            let expanded_fields = expand_tuple_variant_fields_type_defs(fields);

            match expanded_fields.len() {
                0 => {
                    // this case is handles by darling
                    unreachable!()
                }
                1 => {
                    let first = expanded_fields.first().expect("exactly one variant");
                    expand_one_tuple_field_variant_type_defs(span, &ident_str, &first)
                }
                _ => expand_n_tuple_fields_variant_type_defs(span, &ident_str, expanded_fields),
            }
        }
        darling::ast::Style::Struct => {
            let expanded_fields = expand_struct_variant_fields_type_defs(fields);

            match expanded_fields.len() {
                0 => {
                    // this case is handles by darling
                    unreachable!()
                }
                1 => {
                    let first = expanded_fields.first().expect("exactly one variant");
                    expand_one_struct_field_variant_type_defs(span, &ident_str, &first)
                }
                _ => expand_n_struct_fields_variant_type_defs(span, &ident_str, expanded_fields),
            }
        }
    }
}

/// expand a unit variant to a TS definition
/// Example `A`  ->  `"A"`
fn expand_unit_variant_type_defs(span: Span, ident_str: &str) -> TokenStream {
    quote_spanned!(span => format!("\"{}\"", #ident_str))
}

fn expand_struct_variant_fields_type_defs(fields: &Fields<EnumField>) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let ty = &field.ty;
            let span = ty.span();
            let name = field.ident.as_ref().unwrap().to_string();
            quote_spanned!(span => format!("{}: {}", #name, <#ty as remotely_zod::Codegen>::type_def()))
        })
    .collect()
}

fn expand_tuple_variant_fields_type_defs(fields: &Fields<EnumField>) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let ty = &field.ty;
            let span = ty.span();
            quote_spanned!(span => format!("{}", <#ty as remotely_zod::Codegen>::type_def()))
        })
        .collect()
}

/// expand an enum variant with exatly one field to a TS definition
/// Example `A(usize)` ->  `{ A: number }`
fn expand_one_tuple_field_variant_type_defs(
    span: Span,
    ident_str: &str,
    inner: &TokenStream,
) -> TokenStream {
    quote_spanned! {span =>  format!("{{ {}: {} }}", #ident_str, #inner) }
}

/// expand an enum variant with exatly one field to a TS definition
/// Example `A(usize)` ->  `{ A: number }`
fn expand_one_struct_field_variant_type_defs(
    span: Span,
    ident_str: &str,
    inner: &TokenStream,
) -> TokenStream {
    quote_spanned! {span =>  format!("{{ {}: {{ {} }} }}", #ident_str, #inner) }
}

/// expand an enum tuple variant with more than one field to a TS definition
/// Example
/// `A(usize, String)` -> `{ A: [number, string] }`
fn expand_n_tuple_fields_variant_type_defs(
    span: Span,
    ident_str: &str,
    inner: Vec<TokenStream>,
) -> TokenStream {
    let expanded_inner = quote! {
        {
            let inner: std::vec::Vec<String> = vec![#(#inner),*];
            format!("[{}]", inner.join(", "))
        }
    };
    quote_spanned! {span =>  format!("{{ {}: {} }}", #ident_str, #expanded_inner) }
}

/// expand an enum tuple variant with more than one field to a TS definition
/// Example
/// `A{ num: usize, s: String }` -> `{ A: { num: number, s: string } }`
fn expand_n_struct_fields_variant_type_defs(
    span: Span,
    ident_str: &str,
    inner: Vec<TokenStream>,
) -> TokenStream {
    let expanded_inner = quote! {
        {
            let inner: std::vec::Vec<String> = vec![#(#inner),*];
            format!("{}", inner.join(", "))
        }
    };
    quote_spanned! {span =>  format!("{{ {}: {{ {} }} }}", #ident_str, #expanded_inner) }
}
