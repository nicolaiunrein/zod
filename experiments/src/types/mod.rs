mod r#bool;
mod discriminated_union;
mod export;
mod literal;
mod number;
mod object;
mod string;
mod tuple;
mod r#type;
mod union;

use crate::utils::zod_core;

pub use self::r#bool::*;
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

impl ToTokens for ZodTypeAny {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(quote!(#zod_core::types::ZodTypeAny))
    }
}

#[test]
fn ok() {
    assert_eq!(
        quote!(#ZodTypeAny).to_string(),
        quote!(#zod_core::types::ZodTypeAny).to_string()
    )
}
