use std::fmt::Display;

use quote::{quote, ToTokens};
use typed_builder::TypedBuilder;

use crate::{types::Crate, Reference};

use super::{
    literal::ZodLiteral, Ts, Zod, ZodDiscriminatedUnion, ZodNumber, ZodObject, ZodString, ZodTuple,
    ZodUnion,
};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum ZodTypeInner {
    String(ZodString),
    Number(ZodNumber),
    Object(ZodObject),
    Reference(Reference),
    Generic(&'static str),
    Literal(ZodLiteral),
    Union(ZodUnion),
    DiscriminatedUnion(ZodDiscriminatedUnion),
    Tuple(ZodTuple),
}

impl Display for Zod<'_, ZodTypeInner> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ZodTypeInner::String(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Number(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Object(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Reference(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Literal(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Union(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::DiscriminatedUnion(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Tuple(inner) => std::fmt::Display::fmt(&Zod(inner), f),
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
            ZodTypeInner::Reference(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Literal(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Union(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::DiscriminatedUnion(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Tuple(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Generic(inner) => std::fmt::Display::fmt(inner, f),
        }
    }
}

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodType {
    #[builder(setter(strip_bool))]
    pub optional: bool,
    #[builder(setter(into))]
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
            f.write_str(" | undefined")?;
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
            ZodTypeInner::Reference(inner) => (quote!(Arg), quote!(#inner)),
            ZodTypeInner::Literal(inner) => (quote!(Arg), quote!(#inner)),
            ZodTypeInner::Union(inner) => (quote!(Arg), quote!(#inner)),
            ZodTypeInner::DiscriminatedUnion(inner) => (quote!(Arg), quote!(#inner)),
            ZodTypeInner::Tuple(inner) => (quote!(Arg), quote!(#inner)),
            ZodTypeInner::Generic(inner) => (quote!(Generic), quote!(#inner)),
        };

        tokens.extend(quote!(#Crate::types::ZodTypeInner::#variant(#inner)))
    }
}