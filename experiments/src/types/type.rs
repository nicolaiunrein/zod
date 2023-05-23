use std::fmt::Display;

use quote::{quote, ToTokens};

use crate::{types::Crate, Arg};

use super::{Ts, Zod, ZodNumber, ZodObject, ZodString};

pub enum ZodTypeInner {
    String(ZodString),
    Number(ZodNumber),
    Object(ZodObject),
    Arg(Arg),
    Generic(&'static str),
}

impl Display for Zod<'_, ZodTypeInner> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ZodTypeInner::String(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Number(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Object(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Arg(inner) => std::fmt::Display::fmt(&inner.as_zod(), f),
            ZodTypeInner::Generic(inner) => std::fmt::Display::fmt(inner, f),
        }
    }
}

impl Display for Ts<'_, ZodTypeInner> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ZodTypeInner::String(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Number(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Object(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Arg(inner) => std::fmt::Display::fmt(&inner.as_ts(), f),
            ZodTypeInner::Generic(inner) => std::fmt::Display::fmt(inner, f),
        }
    }
}

pub struct ZodType {
    pub optional: bool,
    pub inner: ZodTypeInner,
}

impl Display for Zod<'_, ZodType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Zod(&self.inner).fmt(f)?;
        if self.optional {
            f.write_str(".optional()")?;
        }
        Ok(())
    }
}

impl Display for Ts<'_, ZodType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ts(&self.inner).fmt(f)?;
        if self.optional {
            f.write_str(".optional()")?;
        }
        Ok(())
    }
}

impl<T> From<T> for ZodType
where
    T: Into<ZodTypeInner>,
{
    fn from(value: T) -> Self {
        ZodType {
            optional: false,
            inner: value.into(),
        }
    }
}

impl ToTokens for ZodType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let optional = self.optional;
        let inner = &self.inner;

        tokens.extend(quote!(#Crate::types::ZodType {
            optional: #optional,
            inner: #inner
        }))
    }
}

impl ToTokens for ZodTypeInner {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let (variant, inner) = match self {
            ZodTypeInner::String(inner) => (quote!(String), quote!(#inner)),
            ZodTypeInner::Number(inner) => (quote!(Number), quote!(#inner)),
            ZodTypeInner::Object(inner) => (quote!(Object), quote!(#inner)),
            ZodTypeInner::Arg(inner) => (quote!(Arg), quote!(#inner)),
            ZodTypeInner::Generic(inner) => (quote!(Generic), quote!(#inner)),
        };

        tokens.extend(quote!(#Crate::types::ZodTypeInner::#variant(#inner)))
    }
}
