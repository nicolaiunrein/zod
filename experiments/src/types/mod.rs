mod discriminated_union;
mod export;
mod literal;
mod number;
mod object;
mod string;
mod tuple;
mod r#type;
mod union;

pub use discriminated_union::*;
pub use export::*;
pub use number::*;
pub use object::*;
pub use r#type::*;
pub use string::*;
pub use tuple::*;
pub use union::*;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use std::{fmt::Display, ops::Deref};

use proc_macro2::{Ident, Span};

pub(crate) struct Crate;

impl ToTokens for Crate {
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

pub struct ZodTypeAny;

impl Display for Zod<'_, ZodTypeAny> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.ZodTypeAny")
    }
}

pub struct Zod<'a, T>(pub &'a T);
pub struct Ts<'a, T>(pub &'a T);

impl<T> Deref for Zod<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T> Deref for Ts<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl From<ZodObject> for ZodTypeInner {
    fn from(value: ZodObject) -> Self {
        Self::Object(value)
    }
}

impl ToTokens for ZodTypeAny {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote!(#Crate::types::ZodTypeAny))
    }
}

#[test]
fn ok() {
    assert_eq!(
        quote!(#ZodTypeAny).to_string(),
        quote!(::zod::core::types::ZodTypeAny).to_string()
    )
}
