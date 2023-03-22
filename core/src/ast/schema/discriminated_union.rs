use crate::ast::{Delimited, Formatter};

use super::ObjectSchema;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DiscriminatedUnionSchema {
    key: &'static str,
    variants: &'static [ObjectSchema],
}
impl DiscriminatedUnionSchema {
    pub const fn new(
        key: &'static str,
        variants: &'static [ObjectSchema],
    ) -> DiscriminatedUnionSchema {
        Self { key, variants }
    }
}

impl Formatter for DiscriminatedUnionSchema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let key = self.key;
        f.write_fmt(format_args!("z.discriminatedUnion(\"{key}\", ["))?;

        self.variants
            .iter()
            .comma_separated(f, |f, obj| obj.fmt_zod(f))?;

        f.write_str("])")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.variants
            .iter()
            .fmt_delimited(f, " | ", |f, obj| obj.fmt_ts(f))
    }
}
