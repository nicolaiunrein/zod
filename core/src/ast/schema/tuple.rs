use crate::ast::{Delimited, Formatter};

use super::InlineSchema;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TupleSchema {
    fields: &'static [InlineSchema],
}
impl TupleSchema {
    pub const fn new(fields: &'static [InlineSchema]) -> TupleSchema {
        Self { fields }
    }
}

impl Formatter for TupleSchema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.tuple([")?;
        self.fields
            .iter()
            .comma_separated(f, |f, field| field.fmt_zod(f))?;

        f.write_str("])")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        self.fields
            .iter()
            .comma_separated(f, |f, field| field.fmt_ts(f))?;
        f.write_str("]")?;
        Ok(())
    }
}
