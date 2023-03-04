#![deny(unsafe_code)]

#[cfg(feature = "rpc")]
mod rpc;

#[cfg(feature = "rpc")]
use quote::quote;

mod args;
mod docs;
mod impl_enum;
mod impl_inventory;
mod impl_namespace;
mod impl_struct;

use darling::{ast::Data, FromDeriveInput};
use docs::RustDocs;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::{format_ident, quote_spanned};
use serde_derive_internals::Derive;
use syn::{Ident, Path};

use proc_macro2::Span;

fn get_crate_name() -> String {
    let found_crate = proc_macro_crate::crate_name("zod").unwrap_or_else(|err| {
        proc_macro_error::abort_call_site!("Error: {}", err);
    });

    match found_crate {
        proc_macro_crate::FoundCrate::Itself => String::from("zod"),
        proc_macro_crate::FoundCrate::Name(name) => name,
    }
}

pub(crate) fn get_zod() -> Path {
    let name = get_crate_name();
    let ident = Ident::new(&name, Span::call_site());
    syn::parse_quote!(::#ident)
}

pub(crate) fn get_zod_spanned(span: Span) -> Path {
    let name = get_crate_name();
    let ident = Ident::new(&name, Span::call_site());
    syn::parse_quote_spanned!(span => ::#ident)
}

pub(crate) fn get_private() -> Path {
    let zod = get_zod();
    syn::parse_quote!(#zod::__private)
}

pub(crate) fn get_private_spanned(span: Span) -> Path {
    let zod = get_zod_spanned(span);
    syn::parse_quote_spanned!(span => #zod::__private)
}

#[proc_macro_error]
#[proc_macro_derive(Zod, attributes(zod))]
pub fn zod(input: TokenStream) -> TokenStream {
    let parsed = match syn::parse(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let cx = serde_derive_internals::Ctxt::new();

    let container =
        serde_derive_internals::ast::Container::from_ast(&cx, &parsed, Derive::Deserialize)
            .unwrap();

    cx.check().unwrap();

    let docs = match RustDocs::from_attrs(&parsed.attrs) {
        Ok(docs) => docs,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let input = match args::Input::from_derive_input(&parsed) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let expanded = match input.data.clone() {
        Data::Enum(e) => impl_enum::expand(input, &e, container, docs),
        Data::Struct(e) => impl_struct::expand(input, e, container, docs),
    };
    expanded.into()
}

#[proc_macro_error]
#[proc_macro_derive(Namespace, attributes(namespace))]
pub fn derive_namespace(input: TokenStream) -> TokenStream {
    let parsed = match syn::parse(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };
    let input = match args::NamespaceInput::from_derive_input(&parsed) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let docs = match RustDocs::from_attrs(&parsed.attrs) {
        Ok(docs) => docs,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    impl_namespace::expand(input, docs).into()
}

#[cfg(feature = "rpc")]
#[proc_macro_error]
#[proc_macro_derive(Backend, attributes(rpc))]
pub fn backend(input: TokenStream) -> TokenStream {
    let parsed = match syn::parse(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let input = match rpc::args::BackendInput::from_derive_input(&parsed) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let expanded = match input.data.clone() {
        Data::Enum(_) => unreachable!(),
        Data::Struct(e) => rpc::backend_impl::expand(input, e),
    };
    expanded.into()
}

#[cfg(feature = "rpc")]
#[proc_macro_error]
#[proc_macro_attribute]
pub fn namespace(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let orig = proc_macro2::TokenStream::from(input.clone());

    let ast = syn::parse_macro_input!(input as syn::ItemImpl);
    let extra = rpc::impl_rpc::expand(rpc::args::RpcInput::from_ast(ast));

    let output = quote! {
        #orig

        #extra
    };

    output.into()
}

fn format_ident_for_registration(p: &syn::Path) -> syn::Path {
    let mut segments = p.segments.clone();
    let last = segments.last_mut().unwrap();
    last.ident = format_ident!("__ZodRegister__{}", last.ident);

    syn::Path {
        leading_colon: p.leading_colon,
        segments,
    }
}

/// Prevent duplicate interfaces
fn expand_type_registration(ident: &Ident, ns_path: &Path) -> proc_macro2::TokenStream {
    let register_path = format_ident_for_registration(ns_path);

    quote_spanned! {ident.span() =>
        impl #register_path {
            #[allow(dead_code)]
            #[allow(non_upper_case_globals)]
            const #ident: () = ();
        }
    }
}
