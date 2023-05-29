use std::fmt::Display;

use typed_builder::TypedBuilder;

use crate::{kind, utils::Separated, IoKind};

use super::{Ts, Zod, ZodType, ZodTypeInner};

#[derive(TypedBuilder, Eq, Debug, Clone, Hash)]
pub struct ZodObject<Io> {
    #[builder(default)]
    pub fields: Vec<ZodNamedField<Io>>,
}

impl From<ZodObject<kind::Input>> for ZodObject<kind::EitherIo> {
    fn from(other: ZodObject<kind::Input>) -> Self {
        Self {
            fields: other.fields.into_iter().map(|f| f.into()).collect(),
        }
    }
}

impl From<ZodObject<kind::Output>> for ZodObject<kind::EitherIo> {
    fn from(other: ZodObject<kind::Output>) -> Self {
        Self {
            fields: other.fields.into_iter().map(|f| f.into()).collect(),
        }
    }
}

crate::make_eq!(ZodObject { fields });

impl<Io> Display for Zod<'_, ZodObject<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.fields.is_empty() {
            f.write_str("z.object({})")
        } else {
            let fields = self.fields.iter().map(Zod).collect::<Vec<_>>();
            f.write_fmt(format_args!("z.object({{ {} }})", Separated(", ", &fields)))
        }
    }
}

impl<Io> Display for Ts<'_, ZodObject<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.fields.is_empty() {
            f.write_str("{}")
        } else {
            let fields = self.fields.iter().map(Ts).collect::<Vec<_>>();
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

impl<Io> Display for Zod<'_, ZodNamedField<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.optional {
            f.write_fmt(format_args!(
                "{}: {}.optional()",
                self.name,
                Zod(&self.value)
            ))
        } else {
            f.write_fmt(format_args!("{}: {}", self.name, Zod(&self.value)))
        }
    }
}

impl<Io> Display for Ts<'_, ZodNamedField<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.optional {
            f.write_fmt(format_args!(
                "{}?: {} | undefined",
                self.name,
                Ts(&self.value)
            ))
        } else {
            f.write_fmt(format_args!("{}: {}", self.name, Ts(&self.value)))
        }
    }
}

impl<Io> From<ZodObject<Io>> for ZodTypeInner<Io> {
    fn from(value: ZodObject<Io>) -> Self {
        Self::Object(value)
    }
}

impl From<ZodNamedField<kind::Input>> for ZodNamedField<kind::EitherIo> {
    fn from(other: ZodNamedField<kind::Input>) -> Self {
        Self {
            name: other.name,
            optional: other.optional,
            value: other.value.into(),
        }
    }
}

impl From<ZodNamedField<kind::Output>> for ZodNamedField<kind::EitherIo> {
    fn from(other: ZodNamedField<kind::Output>) -> Self {
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
