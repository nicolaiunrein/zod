use std::fmt::Display;

use quote::{quote, ToTokens};

use crate::utils::zod_core;

use super::ZodTypeInner;
use crate::formatter::{TsFormatter, ZodFormatter};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodString;

impl Display for ZodFormatter<'_, ZodString> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.string()")
    }
}

impl Display for TsFormatter<'_, ZodString> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("string")
    }
}

impl ToTokens for ZodString {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(quote!(#zod_core::types::ZodString))
    }
}

impl<Io> From<ZodString> for ZodTypeInner<Io> {
    fn from(value: ZodString) -> Self {
        ZodTypeInner::String(value)
    }
}
