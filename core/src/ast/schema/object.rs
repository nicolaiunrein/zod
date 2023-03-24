use super::NamedField;
use crate::ast::{Delimited, Formatter};

/// Representation of a `z.object({ ... })`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObjectSchema {
    fields: &'static [NamedField],
}

impl ObjectSchema {
    pub const fn new(fields: &'static [NamedField]) -> Self {
        Self { fields }
    }
}

impl Formatter for ObjectSchema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.object({ ")?;
        self.fields
            .iter()
            .comma_separated(f, |f, field| field.fmt_zod(f))?;

        f.write_str(" })")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{ ")?;
        self.fields
            .iter()
            .comma_separated(f, |f, field| field.fmt_ts(f))?;
        f.write_str(" }")?;
        Ok(())
    }
}
