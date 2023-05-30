use std::fmt::Display;

use typed_builder::TypedBuilder;

use crate::{utils::Separated, IoKind, Kind};

use super::{Ts, Zod, ZodType, ZodTypeInner};

#[derive(TypedBuilder, Eq, Debug, Clone, Hash)]
pub struct ZodUnion<Io> {
    #[builder(default)]
    pub variants: Vec<ZodType<Io>>,
}

impl<Io> Display for Zod<'_, ZodUnion<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(Zod).collect::<Vec<_>>();
        f.write_fmt(format_args!("z.union([{}])", Separated(", ", &variants)))
    }
}

impl<Io> Display for Ts<'_, ZodUnion<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(Ts).collect::<Vec<_>>();
        f.write_fmt(format_args!("{}", Separated(" | ", &variants)))
    }
}

impl<Io> From<ZodUnion<Io>> for ZodTypeInner<Io> {
    fn from(value: ZodUnion<Io>) -> Self {
        ZodTypeInner::Union(value)
    }
}

impl From<ZodUnion<Kind::Input>> for ZodUnion<Kind::EitherIo> {
    fn from(other: ZodUnion<Kind::Input>) -> Self {
        Self {
            variants: other.variants.into_iter().map(|v| v.into()).collect(),
        }
    }
}

impl From<ZodUnion<Kind::Output>> for ZodUnion<Kind::EitherIo> {
    fn from(other: ZodUnion<Kind::Output>) -> Self {
        Self {
            variants: other.variants.into_iter().map(|v| v.into()).collect(),
        }
    }
}

crate::make_eq!(ZodUnion { variants });

#[cfg(test)]
mod test {
    use crate::{
        types::{ZodNumber, ZodString},
        Kind,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn fmt_ok() {
        assert_eq!(
            Zod(&ZodUnion::<Kind::Input>::builder()
                .variants(vec![ZodString.into(), ZodNumber.into()])
                .build())
            .to_string(),
            "z.union([z.string(), z.number()])"
        );

        assert_eq!(
            Ts(&ZodUnion::<Kind::Input>::builder()
                .variants(vec![ZodString.into(), ZodNumber.into()])
                .build())
            .to_string(),
            "string | number"
        );
    }
}
