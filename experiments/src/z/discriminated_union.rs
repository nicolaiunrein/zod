//! TODO: `https://github.com/colinhacks/zod/issues/2106`

use std::fmt::Display;

use typed_builder::TypedBuilder;

use crate::{utils::Separated, IoKind, Kind};

use super::{ZodObject, ZodTypeInner};
use crate::formatter::{TsFormatter, ZodFormatter};

#[derive(TypedBuilder, Eq, Debug, Clone, Hash)]
pub struct ZodDiscriminatedUnion<Io> {
    pub tag: &'static str,
    #[builder(default)]
    pub variants: Vec<ZodObject<Io>>,
}

impl<Io> Display for ZodFormatter<'_, ZodDiscriminatedUnion<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(ZodFormatter).collect::<Vec<_>>();
        f.write_fmt(format_args!(
            "z.discriminatedUnion(\"{tag}\", [{variants}])",
            tag = self.tag,
            variants = Separated(", ", &variants)
        ))
    }
}

impl<Io> Display for TsFormatter<'_, ZodDiscriminatedUnion<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(TsFormatter).collect::<Vec<_>>();
        f.write_fmt(format_args!("{}", Separated(" | ", &variants)))
    }
}

impl<Io> From<ZodDiscriminatedUnion<Io>> for ZodTypeInner<Io> {
    fn from(value: ZodDiscriminatedUnion<Io>) -> Self {
        ZodTypeInner::DiscriminatedUnion(value)
    }
}

impl From<ZodDiscriminatedUnion<Kind::Input>> for ZodDiscriminatedUnion<Kind::EitherIo> {
    fn from(other: ZodDiscriminatedUnion<Kind::Input>) -> Self {
        Self {
            tag: other.tag,
            variants: other.variants.into_iter().map(|v| v.into()).collect(),
        }
    }
}

impl From<ZodDiscriminatedUnion<Kind::Output>> for ZodDiscriminatedUnion<Kind::EitherIo> {
    fn from(other: ZodDiscriminatedUnion<Kind::Output>) -> Self {
        Self {
            tag: other.tag,
            variants: other.variants.into_iter().map(|v| v.into()).collect(),
        }
    }
}

crate::make_eq!(ZodDiscriminatedUnion { tag, variants });

#[cfg(test)]
mod test {
    use crate::TypeExt;
    use crate::{
        z::{ZodNamedField, ZodType},
        Kind,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn fmt_ok() {
        let input = ZodDiscriminatedUnion::<Kind::Input>::builder()
            .tag("abc")
            .variants(vec![
                ZodObject::builder()
                    .fields(vec![ZodNamedField::builder()
                        .name("abc")
                        .value(ZodType::from(<String as TypeExt<Kind::Input>>::inline()))
                        .build()])
                    .build(),
                ZodObject::builder().build(),
            ])
            .build();

        assert_eq!(
            ZodFormatter(&input).to_string(),
            "z.discriminatedUnion(\"abc\", [z.object({ abc: Rs.input.String }), z.object({})])"
        );

        assert_eq!(
            TsFormatter(&input).to_string(),
            "{ abc: Rs.input.String } | {}"
        );
    }
}
