use darling::{ast::Data, FromDeriveInput, FromField};
use proc_macro2::Span;
use proc_macro_error::abort_call_site;
use syn::{parse_quote, Ident, Path, Type};

#[derive(FromDeriveInput)]
pub struct BackendInput {
    pub ident: syn::Ident,
    pub data: Data<darling::util::Ignored, BackendField>,
}

#[derive(FromField, Clone)]
pub struct BackendField {
    pub ident: Option<syn::Ident>,
    pub ty: Type,
}

fn get_crate_name() -> String {
    let found_crate = proc_macro_crate::crate_name("zod").unwrap_or_else(|err| {
        abort_call_site!("Error: {}", err);
    });

    match found_crate {
        proc_macro_crate::FoundCrate::Itself => String::from("zod"),
        proc_macro_crate::FoundCrate::Name(name) => name,
    }
}

pub fn get_zod() -> Path {
    let name = get_crate_name();
    let ident = Ident::new(&name, Span::call_site());
    parse_quote!(::#ident)
}

pub fn get_private() -> Path {
    let zod = get_zod();
    parse_quote!(#zod::rpc::__private)
}
