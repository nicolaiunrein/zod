use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_quote, DataStruct, Generics};

use crate::derive_internals::qualify_ty;
use crate::utils::zod_core;

pub(super) struct StructImpl<Io> {
    pub(crate) ident: Ident,
    pub(crate) generics: Generics,
    pub(crate) data: DataStruct,
    pub(crate) role: Io,
    pub(crate) ns: syn::Path,
    pub(crate) custom_suffix: Option<String>,
}

impl<Io> ToTokens for StructImpl<Io> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ns = &self.ns;
        let ident = &self.ident;
        let name = self.ident.to_string();
        let ns_name = quote_spanned!(ns.span() => <#ns as #zod_core::Namespace>::NAME);
        let custom_suffix = match self.custom_suffix {
            Some(ref suffix) => quote!(Some(#suffix)),
            None => quote!(None),
        };

        let make_export_stmts = |ty: &syn::Type| {
            let qualified_ty = qualify_ty(ty, parse_quote!(#zod_core::IoType));
            quote_spanned!(ty.span() => #qualified_ty::visit_exports(set))
        };

        let exports: Vec<_> = match &self.data.fields {
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| make_export_stmts(&f.ty))
                .collect(),

            syn::Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .map(|f| make_export_stmts(&f.ty))
                .collect(),

            syn::Fields::Unit => todo!(),
        };

        let inner = match &self.data.fields {
            syn::Fields::Named(fields) => impl_zod_object(fields.named.iter().map(|f| (f, false))), //todo
            syn::Fields::Unnamed(fields) => {
                impl_zod_tuple(fields.unnamed.iter().map(|f| (f, false))) // todo
            }
            syn::Fields::Unit => todo!(),
        };

        let arg_names = self
            .generics
            .params
            .iter()
            .map(|p| match p {
                syn::GenericParam::Lifetime(_) => todo!(),
                syn::GenericParam::Type(param) => param.ident.to_string(),
                syn::GenericParam::Const(_) => todo!(),
            })
            .collect::<Vec<_>>();

        tokens.extend(quote!(impl #zod_core::IoType for #ident {
            type Namespace = #ns;
            fn get_ref() -> #zod_core::types::ZodType {
                #zod_core::Reference {
                    name: ::std::string::String::from(#name),
                    ns: ::std::string::String::from(#ns_name),
                    args: vec![#(#arg_names),*]
                }.into()
            }

            fn visit_exports(set: &mut ::std::collections::HashSet<#zod_core::types::ZodExport>) {
                let export = #zod_core::types::ZodExport {
                    ns: ::std::string::String::from(#ns_name),
                    name: ::std::string::String::from(#name),
                    args: &[#(#arg_names),*],
                    value: #zod_core::types::ZodType {
                        optional: false,
                        custom_suffix: #custom_suffix,
                        inner: #inner
                    }
                };

                set.insert(export);
                #(#exports;)*
            }

        }))
    }
}

fn impl_zod_object<'a>(fields: impl Iterator<Item = (&'a syn::Field, bool)>) -> TokenStream2 {
    let fields = fields.map(|(f, optional)| {
        let ident = f.ident.as_ref().expect("named fields");
        let name = ident.to_string();

        let ty = &f.ty;

        let qualified_ty = qualify_ty(ty, parse_quote!(#zod_core::IoType));

        quote_spanned! {
            ty.span() =>
            #zod_core::types::ZodNamedField {
                optional: #optional,
                name: #name,
                value: #zod_core::types::ZodType::from(#qualified_ty::get_ref())
            }
        }
    });

    quote!(#zod_core::types::ZodObject {
        fields: vec![#(#fields),*]
    }.into())
}

fn impl_zod_tuple<'a>(fields: impl Iterator<Item = (&'a syn::Field, bool)>) -> TokenStream2 {
    let fields = fields.map(|(f, optional)| {
        let ty = &f.ty;

        let qualified_ty = qualify_ty(ty, parse_quote!(#zod_core::IoType));

        quote_spanned! {
            ty.span() =>
            #zod_core::types::ZodType {
                optional: #optional,
                ..#qualified_ty::get_ref()
            }
        }
    });

    quote!(#zod_core::types::ZodTuple {
        fields: vec![#(#fields),*]
    }.into())
}
