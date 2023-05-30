//! TODO: `https://github.com/colinhacks/zod/issues/2106`

use std::fmt::Display;

use typed_builder::TypedBuilder;

use crate::{utils::Separated, IoKind, Kind};

use super::{Ts, Zod, ZodObject, ZodTypeInner};

#[derive(TypedBuilder, Eq, Debug, Clone, Hash)]
pub struct ZodDiscriminatedUnion<Io> {
    tag: &'static str,
    #[builder(default)]
    pub variants: Vec<ZodObject<Io>>,
}

impl<Io> Display for Zod<'_, ZodDiscriminatedUnion<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(Zod).collect::<Vec<_>>();
        f.write_fmt(format_args!(
            "z.discriminatedUnion(\"{tag}\", [{variants}])",
            tag = self.tag,
            variants = Separated(", ", &variants)
        ))
    }
}

impl<Io> Display for Ts<'_, ZodDiscriminatedUnion<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(Ts).collect::<Vec<_>>();
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
    use crate::{types::ZodNamedField, Kind, Type};

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
                        .value(<String as Type<Kind::Input>>::get_ref())
                        .build()])
                    .build(),
                ZodObject::builder().build(),
            ])
            .build();
        assert_eq!(
            Zod(&input).to_string(),
            "z.discriminatedUnion(\"abc\", [z.object({ abc: Rs.input.String }), z.object({})])"
        );

        assert_eq!(Ts(&input).to_string(), "{ abc: Rs.input.String } | {}");
    }
}
