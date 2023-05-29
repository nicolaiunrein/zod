use std::fmt::Display;

use typed_builder::TypedBuilder;

use crate::{kind, utils::Separated, IoKind};

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

impl From<ZodUnion<kind::Input>> for ZodUnion<kind::EitherIo> {
    fn from(other: ZodUnion<kind::Input>) -> Self {
        Self {
            variants: other.variants.into_iter().map(|v| v.into()).collect(),
        }
    }
}

impl From<ZodUnion<kind::Output>> for ZodUnion<kind::EitherIo> {
    fn from(other: ZodUnion<kind::Output>) -> Self {
        Self {
            variants: other.variants.into_iter().map(|v| v.into()).collect(),
        }
    }
}

crate::make_eq!(ZodUnion { variants });

#[cfg(test)]
mod test {
    use crate::{
        kind,
        types::{ZodNumber, ZodString},
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn fmt_ok() {
        assert_eq!(
            Zod(&ZodUnion::<kind::Input>::builder()
                .variants(vec![ZodString.into(), ZodNumber.into()])
                .build())
            .to_string(),
            "z.union([z.string(), z.number()])"
        );

        assert_eq!(
            Ts(&ZodUnion::<kind::Input>::builder()
                .variants(vec![ZodString.into(), ZodNumber.into()])
                .build())
            .to_string(),
            "string | number"
        );
    }
}
