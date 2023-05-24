use std::fmt::Display;

use quote::{quote, ToTokens};
use typed_builder::TypedBuilder;

use crate::{types::crate_name, utils::Separated};

use super::{Ts, Zod, ZodType, ZodTypeInner};

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodUnion {
    #[builder(default)]
    pub variants: Vec<ZodType>,
}

impl Display for Zod<'_, ZodUnion> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(Zod).collect::<Vec<_>>();
        f.write_fmt(format_args!("z.union([{}])", Separated(", ", &variants)))
    }
}

impl Display for Ts<'_, ZodUnion> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(Ts).collect::<Vec<_>>();
        f.write_fmt(format_args!("{}", Separated(" | ", &variants)))
    }
}

impl ToTokens for ZodUnion {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variants = &self.variants;

        tokens.extend(quote!(#crate_name::types::ZodUnion {
            variants: vec![#(#variants),*]
        }))
    }
}

impl From<ZodUnion> for ZodTypeInner {
    fn from(value: ZodUnion) -> Self {
        ZodTypeInner::Union(value)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        test_utils::{expand_zod, formatted},
        types::{ZodNumber, ZodString},
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn fmt_ok() {
        assert_eq!(
            Zod(&ZodUnion::builder()
                .variants(vec![ZodString.into(), ZodNumber.into()])
                .build())
            .to_string(),
            "z.union([z.string(), z.number()])"
        );

        assert_eq!(
            Ts(&ZodUnion::builder()
                .variants(vec![ZodString.into(), ZodNumber.into()])
                .build())
            .to_string(),
            "string | number"
        );
    }

    #[test]
    fn to_tokens_ok() {
        let input = ZodUnion::builder()
            .variants(vec![ZodString.into(), ZodNumber.into()])
            .build();

        assert_eq!(
            formatted(quote!(#input)),
            formatted(expand_zod(quote!(crate::types::ZodUnion {
                variants: vec![
                    crate::types::ZodType {
                        optional: false,
                        inner: crate::types::ZodTypeInner::String(crate::types::ZodString)
                    },
                    crate::types::ZodType {
                        optional: false,
                        inner: crate::types::ZodTypeInner::Number(crate::types::ZodNumber)
                    }
                ]
            })))
        )
    }
}
