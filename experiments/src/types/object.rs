use std::fmt::Display;

use typed_builder::TypedBuilder;

use crate::utils::Separated;

use super::{Ts, Zod, ZodType, ZodTypeInner};

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodObject {
    #[builder(default)]
    pub fields: Vec<ZodNamedField>,
}

impl Display for Zod<'_, ZodObject> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.fields.is_empty() {
            f.write_str("z.object({})")
        } else {
            let fields = self.fields.iter().map(Zod).collect::<Vec<_>>();
            f.write_fmt(format_args!("z.object({{ {} }})", Separated(", ", &fields)))
        }
    }
}

impl Display for Ts<'_, ZodObject> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.fields.is_empty() {
            f.write_str("{}")
        } else {
            let fields = self.fields.iter().map(Ts).collect::<Vec<_>>();
            f.write_fmt(format_args!("{{ {} }}", Separated(", ", &fields)))
        }
    }
}

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodNamedField {
    pub name: &'static str,

    #[builder(setter(strip_bool))]
    pub optional: bool,

    #[builder(setter(into))]
    pub value: ZodType,
}

impl Display for Zod<'_, ZodNamedField> {
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

impl Display for Ts<'_, ZodNamedField> {
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

impl From<ZodObject> for ZodTypeInner {
    fn from(value: ZodObject) -> Self {
        Self::Object(value)
    }
}
