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
        let zod = get_zod();
        tokens.extend(quote!(#zod::core))
    }
}

fn get_crate_name() -> String {
    proc_macro_crate::crate_name("zod")
        .map(|found_crate| match found_crate {
            proc_macro_crate::FoundCrate::Itself => String::from("zod"),
            proc_macro_crate::FoundCrate::Name(name) => name,
        })
        .unwrap_or_else(|_| String::from("zod"))
}

pub(crate) fn get_zod() -> syn::Path {
    let name = get_crate_name();
    let ident = Ident::new(&name, Span::call_site());
    syn::parse_quote!(::#ident)
}
