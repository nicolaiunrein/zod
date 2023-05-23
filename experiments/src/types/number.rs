use std::fmt::Display;

use quote::{quote, ToTokens};

use crate::types::Crate;

use super::{Ts, Zod, ZodTypeInner};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodNumber;

impl Display for Zod<'_, ZodNumber> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.number()")
    }
}

impl Display for Ts<'_, ZodNumber> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("number")
    }
}

impl ToTokens for ZodNumber {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!(#Crate::types::ZodNumber))
    }
}

impl From<ZodNumber> for ZodTypeInner {
    fn from(value: ZodNumber) -> Self {
        ZodTypeInner::Number(value)
    }
}
