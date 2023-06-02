use std::fmt::Display;

use quote::{quote, ToTokens};

use crate::z::zod_core;

use super::ZodTypeInner;
use crate::formatter::{TsFormatter, ZodFormatter};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodNumber;

impl Display for ZodFormatter<'_, ZodNumber> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.number()")
    }
}

impl Display for TsFormatter<'_, ZodNumber> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("number")
    }
}

impl ToTokens for ZodNumber {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!(#zod_core::types::ZodNumber))
    }
}

impl<Io> From<ZodNumber> for ZodTypeInner<Io> {
    fn from(value: ZodNumber) -> Self {
        ZodTypeInner::Number(value)
    }
}
