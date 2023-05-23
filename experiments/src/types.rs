use std::fmt::Display;
use typed_builder::TypedBuilder;

use crate::utils::Separated;

pub struct ZodTypeAny;

impl Display for ZodTypeAny {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.ZodTypeAny")
    }
}

pub struct Zod<T>(pub T);
pub struct Ts<T>(pub T);

pub struct ZodString;

impl Display for ZodString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.string()")
    }
}

pub struct ZodNumber;

impl Display for ZodNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.number()")
    }
}

pub enum ZodTypeInner {
    String(ZodString),
    Number(ZodNumber),
    Object(ZodObject),
    Generic(&'static str),
}

impl Display for ZodTypeInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZodTypeInner::String(inner) => std::fmt::Display::fmt(inner, f),
            ZodTypeInner::Number(inner) => std::fmt::Display::fmt(inner, f),
            ZodTypeInner::Object(inner) => std::fmt::Display::fmt(inner, f),
            ZodTypeInner::Generic(inner) => std::fmt::Display::fmt(inner, f),
        }
    }
}

pub struct ZodType {
    pub optional: bool,
    pub inner: ZodTypeInner,
}

impl Display for ZodType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)?;
        if self.optional {
            f.write_str(".optional()")?;
        }
        Ok(())
    }
}

#[derive(TypedBuilder)]
pub struct ZodObject {
    #[builder(default)]
    pub fields: Vec<ZodObjectField>,
}

impl Display for ZodObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "z.object({{ {} }})",
            Separated(",\n", &self.fields)
        ))
    }
}

impl From<ZodObject> for ZodTypeInner {
    fn from(value: ZodObject) -> Self {
        Self::Object(value)
    }
}

impl<T> From<T> for ZodType
where
    T: Into<ZodTypeInner>,
{
    fn from(value: T) -> Self {
        ZodType {
            optional: false,
            inner: value.into(),
        }
    }
}

#[derive(TypedBuilder)]
pub struct ZodObjectField {
    pub name: &'static str,
    #[builder(setter(into))]
    pub value: ZodType,
}

impl Display for ZodObjectField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}: {}", self.name, self.value))
    }
}

#[derive(TypedBuilder)]
pub struct ZodExport {
    pub name: &'static str,
    #[builder(default)]
    pub args: &'static [&'static str],
    #[builder(setter(into))]
    pub value: ZodType,
}

impl Display for ZodExport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.args.is_empty() {
            f.write_fmt(format_args!(
                "export const {name} = {value};",
                name = self.name,
                value = self.value
            ))
        } else {
            let args = self
                .args
                .iter()
                .map(|name| format!("{name}: {ZodTypeAny}"))
                .collect::<Vec<_>>();
            f.write_fmt(format_args!(
                "export const {name} = ({args}) => {value};",
                name = self.name,
                args = Separated(", ", &args),
                value = self.value
            ))
        }
    }
}
