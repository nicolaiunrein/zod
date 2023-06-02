use std::fmt::Display;

use typed_builder::TypedBuilder;

use crate::{utils::Separated, IoKind, Kind};

use super::{ZodType, ZodTypeInner};
use crate::formatter::{TsFormatter, ZodFormatter};

#[derive(TypedBuilder, Eq, Debug, Clone, Hash)]
pub struct ZodObject<Io> {
    #[builder(default)]
    pub fields: Vec<ZodNamedField<Io>>,
}

impl From<ZodObject<Kind::Input>> for ZodObject<Kind::EitherIo> {
    fn from(other: ZodObject<Kind::Input>) -> Self {
        Self {
            fields: other.fields.into_iter().map(|f| f.into()).collect(),
        }
    }
}

impl From<ZodObject<Kind::Output>> for ZodObject<Kind::EitherIo> {
    fn from(other: ZodObject<Kind::Output>) -> Self {
        Self {
            fields: other.fields.into_iter().map(|f| f.into()).collect(),
        }
    }
}

crate::make_eq!(ZodObject { fields });

impl<Io> Display for ZodFormatter<'_, ZodObject<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.fields.is_empty() {
            f.write_str("z.object({})")
        } else {
            let fields = self.fields.iter().map(ZodFormatter).collect::<Vec<_>>();
            f.write_fmt(format_args!("z.object({{ {} }})", Separated(", ", &fields)))
        }
    }
}

impl<Io> Display for TsFormatter<'_, ZodObject<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.fields.is_empty() {
            f.write_str("{}")
        } else {
            let fields = self.fields.iter().map(TsFormatter).collect::<Vec<_>>();
            f.write_fmt(format_args!("{{ {} }}", Separated(", ", &fields)))
        }
    }
}

#[derive(TypedBuilder, Eq, Debug, Clone, Hash)]
pub struct ZodNamedField<Io> {
    pub name: &'static str,

    #[builder(setter(strip_bool))]
    pub optional: bool,

    #[builder(setter(into))]
    pub value: ZodType<Io>,
}

impl<Io> Display for ZodFormatter<'_, ZodNamedField<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.optional {
            f.write_fmt(format_args!(
                "{}: {}.optional()",
                self.name,
                ZodFormatter(&self.value)
            ))
        } else {
            f.write_fmt(format_args!("{}: {}", self.name, ZodFormatter(&self.value)))
        }
    }
}

impl<Io> Display for TsFormatter<'_, ZodNamedField<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.optional {
            f.write_fmt(format_args!(
                "{}?: {} | undefined",
                self.name,
                TsFormatter(&self.value)
            ))
        } else {
            f.write_fmt(format_args!("{}: {}", self.name, TsFormatter(&self.value)))
        }
    }
}

impl<Io> From<ZodObject<Io>> for ZodTypeInner<Io> {
    fn from(value: ZodObject<Io>) -> Self {
        Self::Object(value)
    }
}

impl From<ZodNamedField<Kind::Input>> for ZodNamedField<Kind::EitherIo> {
    fn from(other: ZodNamedField<Kind::Input>) -> Self {
        Self {
            name: other.name,
            optional: other.optional,
            value: other.value.into(),
        }
    }
}

impl From<ZodNamedField<Kind::Output>> for ZodNamedField<Kind::EitherIo> {
    fn from(other: ZodNamedField<Kind::Output>) -> Self {
        Self {
            name: other.name,
            optional: other.optional,
            value: other.value.into(),
        }
    }
}

crate::make_eq!(ZodNamedField {
    name,
    optional,
    value
});
