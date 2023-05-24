//! TODO: `https://github.com/colinhacks/zod/issues/2106`

use std::fmt::Display;

use quote::{quote, ToTokens};
use typed_builder::TypedBuilder;

use crate::{types::crate_name, utils::Separated};

use super::{Ts, Zod, ZodObject, ZodTypeInner};

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodDiscriminatedUnion {
    tag: &'static str,
    #[builder(default)]
    pub variants: Vec<ZodObject>,
}

impl Display for Zod<'_, ZodDiscriminatedUnion> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self
            .variants
            .iter()
            .map(|f| Zod(f, self.context()))
            .collect::<Vec<_>>();
        f.write_fmt(format_args!(
            "z.discriminatedUnion(\"{tag}\", [{variants}])",
            tag = self.tag,
            variants = Separated(", ", &variants)
        ))
    }
}

impl Display for Ts<'_, ZodDiscriminatedUnion> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self
            .variants
            .iter()
            .map(|f| Ts(f, self.context()))
            .collect::<Vec<_>>();
        f.write_fmt(format_args!("{}", Separated(" | ", &variants)))
    }
}

impl ToTokens for ZodDiscriminatedUnion {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variants = &self.variants;
        let tag = self.tag;

        tokens.extend(quote!(#crate_name::types::ZodDiscriminatedUnion {
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
        types::ZodNamedField,
        OutputType,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn fmt_ok() {
        let input = ZodDiscriminatedUnion::builder()
            .tag("abc")
            .variants(vec![
                ZodObject::builder()
                    .fields(vec![ZodNamedField::builder()
                        .name("abc")
                        .value(String::get_output_ref()) //todo
                        .build()])
                    .build()
                    .into(),
                ZodObject::builder().build().into(),
            ])
            .build();
        assert_eq!(
            Zod::io(&input).to_string(),
            "z.discriminatedUnion(\"abc\", [z.object({ abc: Rs.io.String }), z.object({})])"
        );

        assert_eq!(Ts::io(&input).to_string(), "{ abc: Rs.io.String } | {}");
    }

    #[test]
    fn to_tokens_ok() {
        let input = ZodDiscriminatedUnion::builder()
            .tag("abc")
            .variants(vec![
                ZodObject::builder()
                    .fields(vec![ZodNamedField::builder()
                        .name("abc")
                        .value(String::get_output_ref())
                        .build()])
                    .build()
                    .into(),
                ZodObject::builder().build().into(),
            ])
            .build();

        let ref_string = String::get_output_ref();

        assert_eq!(
            formatted(quote!(#input)),
            formatted(expand_zod(quote!(crate::types::ZodDiscriminatedUnion {
                tag: "abc",
                variants: vec![
                    crate::types::ZodObject {
                        fields: vec![crate::types::ZodNamedField {
                            name: "abc",
                            optional: false,
                            value: #ref_string
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
