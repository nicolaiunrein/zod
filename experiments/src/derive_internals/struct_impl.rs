use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_quote, DataStruct, Generics};

use crate::derive_internals::qualify_ty;
use crate::{types::Role, utils::zod_core};

pub(super) struct StructImpl {
    pub(crate) ident: Ident,
    pub(crate) generics: Generics,
    pub(crate) data: DataStruct,
    pub(crate) role: Role,
    pub(crate) ns: String,
}

impl ToTokens for StructImpl {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ns = &self.ns;
        let role = self.role;
        let ident = &self.ident;
        let name = self.ident.to_string();

        let custom_suffix = quote!(None);

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
            syn::Fields::Named(fields) => impl_zod_object(fields),
            syn::Fields::Unnamed(fields) => impl_zod_tuple(fields),
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
            fn get_ref() -> #zod_core::types::ZodType {
                #zod_core::Reference {
                    name: ::std::string::String::from(#name),
                    ns: ::std::string::String::from(#ns),
                    role: #role,
                    args: vec![#(#arg_names),*]
                }.into()
            }

            fn visit_exports(set: &mut ::std::collections::HashSet<#zod_core::types::ZodExport>) {
                let export = #zod_core::types::ZodExport {
                    ns: ::std::string::String::from(#ns),
                    name: ::std::string::String::from(#name),
                    context: #role,
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

fn impl_zod_object(fields: &syn::FieldsNamed) -> TokenStream2 {
    let fields = fields.named.iter().map(|f| {
        let ident = f.ident.as_ref().expect("named fields");
        let name = ident.to_string();
        let optional = false; //todo

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

fn impl_zod_tuple(fields: &syn::FieldsUnnamed) -> TokenStream2 {
    let fields = fields.unnamed.iter().map(|f| {
        let ty = &f.ty;

        let qualified_ty = qualify_ty(ty, parse_quote!(#zod_core::IoType));

        quote_spanned! {
            ty.span() =>
            #zod_core::types::ZodType::from(#qualified_ty::get_ref())
        }
    });

    quote!(#zod_core::types::ZodTuple {
        fields: vec![#(#fields),*]
    }.into())
}
