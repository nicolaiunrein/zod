use crate::{docs::RustDocs, expand_type_registration, get_zod, impl_inventory};

use super::args;
use darling::ast::{Fields, Style};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use serde_derive_internals::ast;
use syn::{spanned::Spanned, Ident, Path};

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
            transparent,
            flatten: attrs.flatten(),
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
        let inventory = impl_inventory::expand(ident, ns_path, name);

        let zod = get_zod();
        if let Some(t) = &self.from_ty {
            quote! {
                impl #zod::ZodType for #ident {
                    fn schema() -> String {
                        <#t as #zod::ZodType>::schema()
                    }

                    fn type_def() -> #zod::TsTypeDef {
                        <#t as #zod::ZodType>::type_def()
                    }

                    fn inline() -> #zod::InlinedType {
                        <#t as #zod::ZodType>::inline()
                    }

                    fn docs() -> Option<&'static str> {
                        Some(#docs)
                    }
                }

                #inventory

                #type_register

            }
        } else {
            quote! {
                impl #zod::ZodType for #ident {
                    fn schema() -> String {
                        #schema
                    }

                    fn type_def() -> #zod::TsTypeDef {
                        #zod::TsTypeDef::Type({ #type_def })
                    }

                    fn inline() -> #zod::InlinedType {
                        #zod::InlinedType::Ref {
                            ns_name: <#ns_path as #zod::Namespace>::NAME,
                            name: #name
                        }
                    }

                    fn docs() -> Option<&'static str> {
                        Some(#docs)
                    }
                }

                #inventory

                #type_register

            }
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
        let ty = qualified_ty(self.ty);

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
        let ty = qualified_ty(self.ty);

        match (self.flatten, &self.name, self.optional, self.transparent) {
            (false, Some(name), false, false) => {
                quote_spanned! {ty.span() =>  format!("{}: {}", #name, #ty::inline()) }
            }
            (false, None, false, false) => {
                // Newtype
                quote_spanned! {ty.span() => #ty::inline().to_string()}
            }
            (false, Some(name), true, false) => {
                quote_spanned! {ty.span() =>  format!("{}?: {} | undefined", #name, #ty::inline()) }
            }
            (false, None, true, false) => {
                // Newtype
                quote_spanned! {ty.span() => format!("{} | undefined", #ty::inline())}
            }

            (false, _, false, true) => {
                // Newtype
                quote_spanned! {ty.span() => #ty::inline().to_string()}
            }
            (false, _, true, true) => {
                quote_spanned! {ty.span() =>  format!("{} | undefined", #ty::inline()) }
            }

            (true, Some(_), false, false) => {
                quote_spanned! {ty.span() =>  format!(" & {}", #ty::inline()) }
            }
            (true, None, false, false) => {
                // Newtype
                quote_spanned! {ty.span() => #ty::inline().to_string()}
            }
            (true, Some(_), true, false) => {
                quote_spanned! {ty.span() =>  format!("& ({} | undefined)", #ty::inline()) }
            }
            (true, None, true, false) => {
                // Newtype
                quote_spanned! {ty.span() => format!("& ({} | undefined)", #ty::inline())}
            }

            (true, _, false, true) => {
                // Newtype
                quote_spanned! {ty.span() => #ty::inline().to_string()}
            }
            (true, _, true, true) => {
                quote_spanned! {ty.span() =>  format!("& ({} | undefined)", #ty::inline()) }
            }
        }
    }
}
