use std::fmt::Display;

use typed_builder::TypedBuilder;

use crate::{Alias, IoKind, Kind, Reference};

use super::{
    literal::ZodLiteral, Ts, Zod, ZodBool, ZodDiscriminatedUnion, ZodNumber, ZodObject, ZodString,
    ZodTuple, ZodUnion,
};

#[derive(Eq, Debug, Clone, Hash)]
pub enum ZodTypeInner<Io> {
    String(ZodString),
    Number(ZodNumber),
    Object(ZodObject<Io>),
    Reference(Reference<Io>),
    Alias(Alias),
    Generic(String),
    Literal(ZodLiteral),
    Union(ZodUnion<Io>),
    DiscriminatedUnion(ZodDiscriminatedUnion<Io>),
    Tuple(ZodTuple<Io>),
    Bool(ZodBool),
}

impl<Io> Display for Zod<'_, ZodTypeInner<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ZodTypeInner::String(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Number(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Object(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Reference(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Alias(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Literal(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Union(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::DiscriminatedUnion(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Tuple(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Bool(inner) => std::fmt::Display::fmt(&Zod(inner), f),
            ZodTypeInner::Generic(inner) => std::fmt::Display::fmt(inner, f),
        }
    }
}

impl<Io> Display for Ts<'_, ZodTypeInner<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            ZodTypeInner::String(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Number(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Object(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Reference(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Alias(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Literal(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Union(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::DiscriminatedUnion(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Tuple(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Bool(inner) => std::fmt::Display::fmt(&Ts(inner), f),
            ZodTypeInner::Generic(inner) => std::fmt::Display::fmt(inner, f),
        }
    }
}

#[derive(TypedBuilder, Eq, Debug, Clone, Hash)]
pub struct ZodType<Io> {
    #[builder(setter(strip_bool))]
    pub optional: bool,

    #[builder(default, setter(strip_option))]
    pub custom_suffix: Option<String>,

    #[builder(setter(into))]
    pub inner: ZodTypeInner<Io>,
}

impl<Io> Display for Zod<'_, ZodType<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Zod(&self.inner).fmt(f)?;
        if self.optional {
            f.write_str(".optional()")?;
        }
        if let Some(ref suffix) = self.custom_suffix {
            if !suffix.starts_with('.') {
                f.write_str(".")?;
            }
            f.write_str(suffix)?;
        }
        Ok(())
    }
}

impl<Io> Display for Ts<'_, ZodType<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ts(&self.inner).fmt(f)?;
        if self.optional {
            f.write_str(" | undefined")?;
        }
        Ok(())
    }
}

impl<T, Io> From<T> for ZodType<Io>
where
    T: Into<ZodTypeInner<Io>>,
{
    fn from(value: T) -> Self {
        ZodType {
            optional: false,
            custom_suffix: None,
            inner: value.into(),
        }
    }
}

impl From<ZodType<Kind::Input>> for ZodType<Kind::EitherIo> {
    fn from(other: ZodType<Kind::Input>) -> Self {
        ZodType {
            optional: other.optional.into(),
            custom_suffix: other.custom_suffix.into(),
            inner: other.inner.into(),
        }
    }
}

impl From<ZodType<Kind::Output>> for ZodType<Kind::EitherIo> {
    fn from(other: ZodType<Kind::Output>) -> Self {
        ZodType {
            optional: other.optional.into(),
            custom_suffix: other.custom_suffix.into(),
            inner: other.inner.into(),
        }
    }
}

crate::make_eq!(ZodType {
    optional,
    custom_suffix,
    inner
});

impl From<ZodTypeInner<Kind::Input>> for ZodTypeInner<Kind::EitherIo> {
    fn from(other: ZodTypeInner<Kind::Input>) -> Self {
        match other {
            ZodTypeInner::String(inner) => ZodTypeInner::String(inner.into()),
            ZodTypeInner::Number(inner) => ZodTypeInner::Number(inner.into()),
            ZodTypeInner::Object(inner) => ZodTypeInner::Object(inner.into()),
            ZodTypeInner::Reference(inner) => ZodTypeInner::Reference(inner.into()),
            ZodTypeInner::Alias(inner) => ZodTypeInner::Alias(inner),
            ZodTypeInner::Generic(inner) => ZodTypeInner::Generic(inner.into()),
            ZodTypeInner::Literal(inner) => ZodTypeInner::Literal(inner.into()),
            ZodTypeInner::Union(inner) => ZodTypeInner::Union(inner.into()),
            ZodTypeInner::DiscriminatedUnion(inner) => {
                ZodTypeInner::DiscriminatedUnion(inner.into())
            }
            ZodTypeInner::Tuple(inner) => ZodTypeInner::Tuple(inner.into()),
            ZodTypeInner::Bool(inner) => ZodTypeInner::Bool(inner.into()),
        }
    }
}

impl From<ZodTypeInner<Kind::Output>> for ZodTypeInner<Kind::EitherIo> {
    fn from(other: ZodTypeInner<Kind::Output>) -> Self {
        match other {
            ZodTypeInner::String(inner) => ZodTypeInner::String(inner.into()),
            ZodTypeInner::Number(inner) => ZodTypeInner::Number(inner.into()),
            ZodTypeInner::Object(inner) => ZodTypeInner::Object(inner.into()),
            ZodTypeInner::Reference(inner) => ZodTypeInner::Reference(inner.into()),
            ZodTypeInner::Alias(inner) => ZodTypeInner::Alias(inner),
            ZodTypeInner::Generic(inner) => ZodTypeInner::Generic(inner.into()),
            ZodTypeInner::Literal(inner) => ZodTypeInner::Literal(inner.into()),
            ZodTypeInner::Union(inner) => ZodTypeInner::Union(inner.into()),
            ZodTypeInner::DiscriminatedUnion(inner) => {
                ZodTypeInner::DiscriminatedUnion(inner.into())
            }
            ZodTypeInner::Tuple(inner) => ZodTypeInner::Tuple(inner.into()),
            ZodTypeInner::Bool(inner) => ZodTypeInner::Bool(inner.into()),
        }
    }
}

impl<A, B> PartialEq<ZodTypeInner<A>> for ZodTypeInner<B> {
    fn eq(&self, other: &ZodTypeInner<A>) -> bool {
        match (self, other) {
            (ZodTypeInner::String(a), ZodTypeInner::String(b)) => a == b,
            (ZodTypeInner::Number(a), ZodTypeInner::Number(b)) => a == b,
            (ZodTypeInner::Object(a), ZodTypeInner::Object(b)) => a == b,
            (ZodTypeInner::Reference(a), ZodTypeInner::Reference(b)) => a == b,
            (ZodTypeInner::Alias(a), ZodTypeInner::Alias(b)) => a == b,
            (ZodTypeInner::Generic(a), ZodTypeInner::Generic(b)) => a == b,
            (ZodTypeInner::Literal(a), ZodTypeInner::Literal(b)) => a == b,
            (ZodTypeInner::Union(a), ZodTypeInner::Union(b)) => a == b,
            (ZodTypeInner::DiscriminatedUnion(a), ZodTypeInner::DiscriminatedUnion(b)) => a == b,
            (ZodTypeInner::Tuple(a), ZodTypeInner::Tuple(b)) => a == b,
            (ZodTypeInner::Bool(a), ZodTypeInner::Bool(b)) => a == b,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Kind;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn custom_ok() {
        let input = ZodType::<Kind::Input>::builder()
            .custom_suffix(String::from(".min(24)"))
            .inner(ZodNumber)
            .build();

        assert_eq!(Zod(&input).to_string(), "z.number().min(24)");
    }
}
