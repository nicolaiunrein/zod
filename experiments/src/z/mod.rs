mod r#bool;
mod discriminated_union;
mod literal;
mod number;
mod object;
mod string;
mod tuple;
mod r#type;
mod union;

use crate::formatter::ZodFormatter;
use crate::utils::zod_core;

pub use self::r#bool::*;
pub use discriminated_union::*;
pub use literal::*;
pub use number::*;
pub use object::*;
pub use r#type::*;
pub use string::*;
pub use tuple::*;
pub use union::*;

use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

use std::fmt::Display;

pub struct ZodTypeAny;

impl Display for ZodFormatter<'_, ZodTypeAny> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.ZodTypeAny")
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
