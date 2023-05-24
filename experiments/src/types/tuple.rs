use std::fmt::Display;

use quote::{quote, ToTokens};
use typed_builder::TypedBuilder;

use crate::{types::crate_name, utils::Separated};

use super::{Ts, Zod, ZodType, ZodTypeInner};

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodTuple {
    #[builder(default)]
    pub variants: Vec<ZodType>,
}

impl Display for Zod<'_, ZodTuple> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(|f| Zod(f)).collect::<Vec<_>>();
        f.write_fmt(format_args!("z.tuple([{}])", Separated(", ", &variants)))
    }
}

impl Display for Ts<'_, ZodTuple> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(|f| Ts(f)).collect::<Vec<_>>();
        f.write_fmt(format_args!("[{}]", Separated(", ", &variants)))
    }
}

impl ToTokens for ZodTuple {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variants = &self.variants;

        tokens.extend(quote!(#crate_name::types::ZodTuple {
            variants: vec![#(#variants),*]
        }))
    }
}

impl From<ZodTuple> for ZodTypeInner {
    fn from(value: ZodTuple) -> Self {
        ZodTypeInner::Tuple(value)
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
            Zod(&ZodTuple::builder()
                .variants(vec![ZodString.into(), ZodNumber.into()])
                .build())
            .to_string(),
            "z.tuple([z.string(), z.number()])"
        );

        assert_eq!(
            Ts(&ZodTuple::builder()
                .variants(vec![ZodString.into(), ZodNumber.into()])
                .build())
            .to_string(),
            "[string, number]"
        );
    }

    #[test]
    fn to_tokens_ok() {
        let input = ZodTuple::builder()
            .variants(vec![ZodString.into(), ZodNumber.into()])
            .build();

        assert_eq!(
            formatted(quote!(#input)),
            formatted(expand_zod(quote!(crate::types::ZodTuple {
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
