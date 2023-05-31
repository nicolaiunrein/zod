use std::fmt::Display;

use typed_builder::TypedBuilder;

use crate::{utils::Separated, IoKind, Kind};

use super::{ZodType, ZodTypeInner};
use crate::formatter::{TsFormatter, ZodFormatter};

#[derive(TypedBuilder, Eq, Debug, Clone, Hash)]
pub struct ZodTuple<Io> {
    #[builder(default)]
    pub fields: Vec<ZodType<Io>>,
}

impl<Io> Display for ZodFormatter<'_, ZodTuple<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.fields.iter().map(ZodFormatter).collect::<Vec<_>>();
        f.write_fmt(format_args!("z.tuple([{}])", Separated(", ", &variants)))
    }
}

impl<Io> Display for TsFormatter<'_, ZodTuple<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.fields.iter().map(TsFormatter).collect::<Vec<_>>();
        f.write_fmt(format_args!("[{}]", Separated(", ", &variants)))
    }
}

impl<Io> From<ZodTuple<Io>> for ZodTypeInner<Io> {
    fn from(value: ZodTuple<Io>) -> Self {
        ZodTypeInner::Tuple(value)
    }
}

impl From<ZodTuple<Kind::Input>> for ZodTuple<Kind::EitherIo> {
    fn from(other: ZodTuple<Kind::Input>) -> Self {
        Self {
            fields: other.fields.into_iter().map(|f| f.into()).collect(),
        }
    }
}

impl From<ZodTuple<Kind::Output>> for ZodTuple<Kind::EitherIo> {
    fn from(other: ZodTuple<Kind::Output>) -> Self {
        Self {
            fields: other.fields.into_iter().map(|f| f.into()).collect(),
        }
    }
}

crate::make_eq!(ZodTuple { fields });

#[cfg(test)]
mod test {
    use crate::{z, Kind};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn fmt_ok() {
        assert_eq!(
            ZodFormatter(
                &ZodTuple::<Kind::Input>::builder()
                    .fields(vec![z::ZodString.into(), z::ZodNumber.into()])
                    .build()
            )
            .to_string(),
            "z.tuple([z.string(), z.number()])"
        );

        assert_eq!(
            TsFormatter(
                &ZodTuple::<Kind::Input>::builder()
                    .fields(vec![z::ZodString.into(), z::ZodNumber.into()])
                    .build()
            )
            .to_string(),
            "[string, number]"
        );
    }
}
