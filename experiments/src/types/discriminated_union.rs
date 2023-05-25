//! TODO: `https://github.com/colinhacks/zod/issues/2106`

use std::fmt::Display;

use typed_builder::TypedBuilder;

use crate::utils::Separated;

use super::{Ts, Zod, ZodObject, ZodTypeInner};

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodDiscriminatedUnion {
    tag: &'static str,
    #[builder(default)]
    pub variants: Vec<ZodObject>,
}

impl Display for Zod<'_, ZodDiscriminatedUnion> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(Zod).collect::<Vec<_>>();
        f.write_fmt(format_args!(
            "z.discriminatedUnion(\"{tag}\", [{variants}])",
            tag = self.tag,
            variants = Separated(", ", &variants)
        ))
    }
}

impl Display for Ts<'_, ZodDiscriminatedUnion> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(Ts).collect::<Vec<_>>();
        f.write_fmt(format_args!("{}", Separated(" | ", &variants)))
    }
}

impl From<ZodDiscriminatedUnion> for ZodTypeInner {
    fn from(value: ZodDiscriminatedUnion) -> Self {
        ZodTypeInner::DiscriminatedUnion(value)
    }
}

#[cfg(test)]
mod test {
    use crate::{types::ZodNamedField, OutputType};

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
            Zod(&input).to_string(),
            "z.discriminatedUnion(\"abc\", [z.object({ abc: Rs.io.String }), z.object({})])"
        );

        assert_eq!(Ts(&input).to_string(), "{ abc: Rs.io.String } | {}");
    }
}
