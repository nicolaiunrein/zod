use std::fmt::Display;

use quote::{quote, ToTokens};
use typed_builder::TypedBuilder;

use crate::{types::Crate, utils::Separated, Reference};

use super::{Ts, Zod, ZodTypeInner};

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodObject {
    #[builder(default)]
    pub fields: Vec<ZodNamedField>,
}

impl Display for Zod<'_, ZodObject> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.fields.is_empty() {
            f.write_str("z.object({})")
        } else {
            let fields = self.fields.iter().map(|f| Zod(f)).collect::<Vec<_>>();
            f.write_fmt(format_args!("z.object({{ {} }})", Separated(", ", &fields)))
        }
    }
}

impl Display for Ts<'_, ZodObject> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.fields.is_empty() {
            f.write_str("{}")
        } else {
            let fields = self.fields.iter().map(|f| Ts(f)).collect::<Vec<_>>();
            f.write_fmt(format_args!("{{ {} }}", Separated(", ", &fields)))
        }
    }
}

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodNamedField {
    pub name: &'static str,

    #[builder(setter(strip_bool))]
    pub optional: bool,

    #[builder(setter(into))]
    pub value: Reference,
}

impl Display for Zod<'_, ZodNamedField> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}: {}", self.name, Zod(&self.value)))
    }
}

impl Display for Ts<'_, ZodNamedField> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.optional {
            f.write_fmt(format_args!("{}?: {}", self.name, Ts(&self.value)))
        } else {
            f.write_fmt(format_args!("{}: {}", self.name, Ts(&self.value)))
        }
    }
}

impl ToTokens for ZodObject {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fields = &self.fields;
        tokens.extend(quote!(#Crate::types::ZodObject {
            fields: vec![#(#fields),*],
        }))
    }
}

impl ToTokens for ZodNamedField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.name;
        let value = &self.value;
        let optional = self.optional;

        tokens.extend(quote!(#Crate::types::ZodNamedField {
            name: #name,
            optional: #optional,
            value: #value
        }))
    }
}

impl From<ZodObject> for ZodTypeInner {
    fn from(value: ZodObject) -> Self {
        Self::Object(value)
    }
}
