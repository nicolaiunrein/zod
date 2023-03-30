use proc_macro2::Span;
use syn::{Ident, Path};

fn get_crate_name() -> String {
    proc_macro_crate::crate_name("zod")
        .map(|found_crate| match found_crate {
            proc_macro_crate::FoundCrate::Itself => String::from("zod"),
            proc_macro_crate::FoundCrate::Name(name) => name,
        })
        .unwrap_or_else(|_| String::from("zod"))
}

pub(crate) fn get_zod() -> Path {
    let name = get_crate_name();
    let ident = Ident::new(&name, Span::call_site());
    syn::parse_quote!(::#ident)
}
