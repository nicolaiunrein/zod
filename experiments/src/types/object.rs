use std::fmt::Display;

use quote::{quote, ToTokens};
use typed_builder::TypedBuilder;

use crate::{types::Crate, utils::Separated};

use super::{Ts, Zod, ZodType};

#[derive(TypedBuilder)]
pub struct ZodObject {
    #[builder(default)]
    pub fields: Vec<ZodObjectField>,
}

impl Display for Zod<'_, ZodObject> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fields = self.fields.iter().map(|f| Zod(f)).collect::<Vec<_>>();
        f.write_fmt(format_args!("z.object({{ {} }})", Separated(", ", &fields)))
    }
}

impl Display for Ts<'_, ZodObject> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fields = self.fields.iter().map(|f| Ts(f)).collect::<Vec<_>>();
        f.write_fmt(format_args!("{{ {} }}", Separated(", ", &fields)))
    }
}

#[derive(TypedBuilder)]
pub struct ZodObjectField {
    pub name: &'static str,
    #[builder(setter(into))]
    pub value: ZodType,
}

impl Display for Zod<'_, ZodObjectField> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}: {}", self.name, Zod(&self.value)))
    }
}

impl Display for Ts<'_, ZodObjectField> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.value.optional {
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

impl ToTokens for ZodObjectField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.name;
        let value = &self.value;

        tokens.extend(quote!(#Crate::types::ZodObjectField {
            name: #name,
            value: #value
        }))
    }
}
