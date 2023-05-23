//! TODO: https://github.com/colinhacks/zod/issues/2106

use std::fmt::Display;

use quote::{quote, ToTokens};
use typed_builder::TypedBuilder;

use crate::{types::Crate, utils::Separated};

use super::{Ts, Zod, ZodObject, ZodTypeInner};

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodDiscriminatedUnion {
    tag: &'static str,
    #[builder(default)]
    pub variants: Vec<ZodObject>,
}

impl Display for Zod<'_, ZodDiscriminatedUnion> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(|f| Zod(f)).collect::<Vec<_>>();
        f.write_fmt(format_args!(
            "z.discriminatedUnion(\"{tag}\", [{variants}])",
            tag = self.tag,
            variants = Separated(", ", &variants)
        ))
    }
}

impl Display for Ts<'_, ZodDiscriminatedUnion> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(|f| Ts(f)).collect::<Vec<_>>();
        f.write_fmt(format_args!("{}", Separated(" | ", &variants)))
    }
}

impl ToTokens for ZodDiscriminatedUnion {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variants = &self.variants;
        let tag = self.tag;

        tokens.extend(quote!(#Crate::types::ZodDiscriminatedUnion {
            tag: #tag,
            variants: vec![#(#variants),*]
        }))
    }
}

impl From<ZodDiscriminatedUnion> for ZodTypeInner {
    fn from(value: ZodDiscriminatedUnion) -> Self {
        ZodTypeInner::DiscriminatedUnion(value)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        test_utils::{expand_zod, formatted},
        types::{ZodObjectField, ZodString},
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn fmt_ok() {
        let input = ZodDiscriminatedUnion::builder()
            .tag("abc")
            .variants(vec![
                ZodObject::builder()
                    .fields(vec![ZodObjectField::builder()
                        .name("abc")
                        .value(ZodString)
                        .build()])
                    .build()
                    .into(),
                ZodObject::builder().build().into(),
            ])
            .build();
        assert_eq!(
            Zod(&input).to_string(),
            "z.discriminatedUnion(\"abc\", [z.object({ abc: z.string() }), z.object({})])"
        );

        assert_eq!(Ts(&input).to_string(), "{ abc: string } | {}");
    }

    #[test]
    fn to_tokens_ok() {
        let input = ZodDiscriminatedUnion::builder()
            .tag("abc")
            .variants(vec![
                ZodObject::builder()
                    .fields(vec![ZodObjectField::builder()
                        .name("abc")
                        .value(ZodString)
                        .build()])
                    .build()
                    .into(),
                ZodObject::builder().build().into(),
            ])
            .build();

        assert_eq!(
            formatted(quote!(#input)),
            formatted(expand_zod(quote!(crate::types::ZodDiscriminatedUnion {
                tag: "abc",
                variants: vec![
                    crate::types::ZodObject {
                        fields: vec![crate::types::ZodObjectField {
                            name: "abc",
                            value: crate::types::ZodType {
                                optional: false,
                                inner: crate::types::ZodTypeInner::String(crate::types::ZodString)
                            }
                        }],
                    },
                    crate::types::ZodObject {
                        // force new line for comma
                        fields: vec![],
                    }
                ]
            })))
        )
    }
}
