use crate::{docs::RustDocs, expand_type_registration};

use super::args;
use darling::ast::{Fields, Style};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use serde_derive_internals::ast;
use syn::{spanned::Spanned, Ident, Path};

fn qualified_ty(ty: &syn::Type) -> proc_macro2::TokenStream {
    quote!(<#ty as ::zod::ZodType>)
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

    let style = fields.style;

    let fields = fields
        .iter()
        .zip(fields_ast.iter().map(|f| &f.attrs))
        .filter(|(_, attrs)| !attrs.skip_deserializing())
        .map(|(args::StructField { ty, ident }, attrs)| StructField {
            ty: &ty,
            name: ident.as_ref().map(|_| attrs.name().deserialize_name()),
            optional: !attrs.default().is_none(),
            transparent,
            flatten: attrs.flatten(),
        })
        .collect();

    let struct_def = Struct {
        transparent,
        ns_path,
        name,
        docs,
        ident,
        fields,
        style,
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
}

impl<'a> Struct<'a> {
    fn expand(&self) -> TokenStream {
        let schema = self.expand_schema();
        let type_def = self.expand_type_def();
        let ident = &self.ident;
        let ns_path = &self.ns_path;
        let name = &self.name;
        let docs = &self.docs;

        let type_register = expand_type_registration(ident, ns_path);

        quote! {
            impl ::zod::ZodType for #ident {
                fn schema() -> String {
                    #schema
                }

                fn type_def() -> String {
                    #type_def
                }

                fn type_name() -> String {
                    format!("{}.{}", <#ns_path as ::zod::Namespace>::NAME, #name)
                }

                fn docs() -> Option<&'static str> {
                    Some(#docs)
                }
            }

            #type_register

        }
    }

    fn expand_type_def(&self) -> TokenStream {
        let (flat_fields, fields) = self.fields.iter().partition::<Vec<_>, _>(|f| f.flatten);

        match (self.transparent, self.style) {
            (true, _) => fields
                .into_iter()
                .next()
                .expect("At least one field")
                .expand_type_defs(),

            (false, Style::Tuple) => match fields.len() {
                0 => unreachable!("handled by darling"),
                1 => fields
                    .into_iter()
                    .next()
                    .or_else(|| flat_fields.into_iter().next())
                    .expect("Newtype")
                    .expand_type_defs(),
                _ => {
                    let fields = fields.into_iter().map(|f| f.expand_type_defs());

                    quote! {
                        let fields: Vec<String> = vec![#(#fields),*];
                        format!("[{}]", fields.join(", "))
                    }
                }
            },

            (false, Style::Struct) => {
                let fields = fields.into_iter().map(|f| f.expand_type_defs());
                let flat_fields = flat_fields.into_iter().map(|f| f.expand_type_defs());

                quote! {
                    let fields: Vec<String> = vec![#(#fields),*];
                    let extensions: Vec<String> = vec![#(#flat_fields),*];
                    format!("{{{}}}{}", fields.join(",\n"), extensions.join(""))
                }
            }

            (false, Style::Unit) => unreachable!(),
        }
    }

    fn expand_schema(&self) -> TokenStream {
        let (flat_fields, fields) = self.fields.iter().partition::<Vec<_>, _>(|f| f.flatten);

        match (self.transparent, self.style) {
            (true, _) => fields
                .into_iter()
                .next()
                .expect("At least one field")
                .expand_schema(),

            (false, Style::Tuple) => match fields.len() {
                0 => unreachable!("handled by darling"),
                1 => fields
                    .into_iter()
                    .next()
                    .or_else(|| flat_fields.into_iter().next())
                    .expect("Newtype")
                    .expand_schema(),
                _ => {
                    // make sure all fields followed an optional field are also optional
                    if let Some(f) = fields
                        .iter()
                        .skip_while(|f| !f.optional)
                        .find(|f| !f.optional)
                    {
                        abort!(f.ty.span(), "zod: non-default field follows default field")
                    }

                    let fields = fields.into_iter().map(|f| f.expand_schema());

                    quote! {
                        let fields: Vec<String> = vec![#(#fields),*];
                        format!("z.tuple([{}])", fields.join(", "))
                    }
                }
            },

            (false, Style::Struct) => {
                let fields = fields.into_iter().map(|f| f.expand_schema());
                let flat_fields = flat_fields.into_iter().map(|f| f.expand_schema());

                quote! {
                    let fields: Vec<String> = vec![#(#fields),*];
                    let extensions: Vec<String> = vec![#(#flat_fields),*];
                    format!("z.object({{{}}}){}", fields.join(",\n"), extensions.join(""))
                }
            }

            (false, Style::Unit) => unreachable!(),
        }
    }
}

struct StructField<'a> {
    name: Option<String>,
    ty: &'a syn::Type,
    optional: bool,
    transparent: bool,
    flatten: bool,
}

impl<'a> StructField<'a> {
    fn expand_schema(&self) -> TokenStream {
        let maybe_optional = self.expand_optional_schema();
        let ty = qualified_ty(&self.ty);

        match (self.flatten, &self.name, self.transparent) {
            (false, Some(name), false) => {
                quote_spanned! {ty.span() =>  format!("{}: {}{},", #name, #ty::schema(), #maybe_optional) }
            }
            (false, Some(_), true) => {
                quote_spanned! {ty.span() =>  format!("{}{}", #ty::schema(), #maybe_optional) }
            }
            (false, None, _) => {
                quote_spanned! { ty.span() => format!("{}{}", #ty::schema(), #maybe_optional) }
            }

            (true, Some(_), false) => {
                quote_spanned! {ty.span() =>  format!(".extend({}{})", #ty::schema(), #maybe_optional) }
            }
            (true, Some(_), true) => {
                quote_spanned! {ty.span() =>  format!(".extend({}{})", #ty::schema(), #maybe_optional) }
            }
            (true, None, _) => {
                // Newtype
                quote_spanned! { ty.span() => format!(".extend({}{})", #ty::schema(), #maybe_optional) }
            }
        }
    }

    fn expand_optional_schema(&self) -> TokenStream {
        if self.optional {
            quote!(".optional()")
        } else {
            quote!("")
        }
    }

    fn expand_type_defs(&self) -> TokenStream {
        let ty = qualified_ty(&self.ty);

        match (self.flatten, &self.name, self.optional, self.transparent) {
            (false, Some(name), false, false) => {
                quote_spanned! {ty.span() =>  format!("{}: {}", #name, #ty::type_name()) }
            }
            (false, None, false, false) => {
                // Newtype
                quote_spanned! {ty.span() => #ty::type_name()}
            }
            (false, Some(name), true, false) => {
                quote_spanned! {ty.span() =>  format!("{}?: {} | undefined", #name, #ty::type_name()) }
            }
            (false, None, true, false) => {
                // Newtype
                quote_spanned! {ty.span() => format!("{} | undefined", #ty::type_name())}
            }

            (false, _, false, true) => {
                // Newtype
                quote_spanned! {ty.span() => #ty::type_name()}
            }
            (false, _, true, true) => {
                quote_spanned! {ty.span() =>  format!("{} | undefined", #ty::type_name()) }
            }

            (true, Some(_), false, false) => {
                quote_spanned! {ty.span() =>  format!(" & {}", #ty::type_name()) }
            }
            (true, None, false, false) => {
                // Newtype
                quote_spanned! {ty.span() => #ty::type_name()}
            }
            (true, Some(_), true, false) => {
                quote_spanned! {ty.span() =>  format!("& ({} | undefined)", #ty::type_name()) }
            }
            (true, None, true, false) => {
                // Newtype
                quote_spanned! {ty.span() => format!("& ({} | undefined)", #ty::type_name())}
            }

            (true, _, false, true) => {
                // Newtype
                quote_spanned! {ty.span() => #ty::type_name()}
            }
            (true, _, true, true) => {
                quote_spanned! {ty.span() =>  format!("& ({} | undefined)", #ty::type_name()) }
            }
        }
    }
}
