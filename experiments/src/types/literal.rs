use std::fmt::Display;

use quote::{quote, ToTokens};

use crate::types::crate_name;

use super::{Ts, Zod, ZodTypeInner};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum ZodLiteral {
    String(&'static str),
    Number(isize),
    Bool(bool),
}
impl Display for Zod<'_, ZodLiteral> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ZodLiteral::String(inner) => f.write_fmt(format_args!("z.literal(\"{inner}\")")),
            ZodLiteral::Number(inner) => f.write_fmt(format_args!("z.literal({inner})")),
            ZodLiteral::Bool(inner) => f.write_fmt(format_args!("z.literal({inner})")),
        }
    }
}

impl Display for Ts<'_, ZodLiteral> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ZodLiteral::String(inner) => f.write_fmt(format_args!("\"{inner}\"")),
            ZodLiteral::Number(inner) => f.write_fmt(format_args!("{inner}")),
            ZodLiteral::Bool(inner) => f.write_fmt(format_args!("{inner}")),
        }
    }
}

impl ToTokens for ZodLiteral {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(match self {
            ZodLiteral::String(inner) => quote!(#crate_name::types::Literal::String(#inner)),
            ZodLiteral::Number(inner) => quote!(#crate_name::types::Literal::Number(#inner)),
            ZodLiteral::Bool(inner) => quote!(#crate_name::types::Literal::Bool(#inner)),
        });
    }
}

impl From<ZodLiteral> for ZodTypeInner {
    fn from(value: ZodLiteral) -> Self {
        Self::Literal(value)
    }
}
