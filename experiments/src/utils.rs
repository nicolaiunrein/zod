use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use std::fmt;

#[derive(Clone, Debug)]
pub struct Separated<'a, Sep: fmt::Display, Item: fmt::Display + 'a>(pub Sep, pub &'a [Item]);

impl<'a, Sep, Item> fmt::Display for Separated<'a, Sep, Item>
where
    Sep: fmt::Display,
    Item: fmt::Display + 'a,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut iterator = self.1.into_iter();
        if let Some(x) = iterator.next() {
            write!(f, "{}", x)?;
            for item in iterator {
                write!(f, "{}{}", self.0, item)?;
            }
        }
        Ok(())
    }
}

#[allow(non_camel_case_types)]
pub(crate) struct crate_name;

impl ToTokens for crate_name {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let p: syn::Path = get_crate_name("zod_core")
            .map(|name| {
                let ident = Ident::new(&name, Span::call_site());
                syn::parse_quote!(::#ident)
            })
            .or_else(|| {
                get_crate_name("zod").map(|name| {
                    let ident = Ident::new(&name, Span::call_site());
                    syn::parse_quote!(::#ident::core)
                })
            })
            .unwrap_or_else(|| syn::parse_quote!(::zod_core));

        tokens.extend(quote!(#p))
    }
}

fn get_crate_name(name: &'static str) -> Option<String> {
    proc_macro_crate::crate_name(name)
        .map(|found_crate| match found_crate {
            proc_macro_crate::FoundCrate::Itself => String::from(name),
            proc_macro_crate::FoundCrate::Name(name) => name,
        })
        .ok()
}
