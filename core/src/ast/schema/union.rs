use crate::ast::{Delimited, Formatter};

use super::Ref;

/// Representation of a `z.union([ ... ])`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnionSchema {
    variants: &'static [Ref],
}

impl UnionSchema {
    pub const fn new(variants: &'static [Ref]) -> UnionSchema {
        Self { variants }
    }
}

impl Formatter for UnionSchema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.union([")?;
        self.variants
            .iter()
            .comma_separated(f, |f, field| field.fmt_zod(f))?;

        f.write_str("])")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.variants
            .iter()
            .fmt_delimited(f, " | ", |f, field| field.fmt_ts(f))?;
        Ok(())
    }
}
