use crate::ast::Formatter;

use super::Ref;

/// An type alias
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NewtypeSchema {
    inner: &'static Ref,
    optional: bool,
}
impl NewtypeSchema {
    pub const fn new(inner: &'static Ref, optional: bool) -> NewtypeSchema {
        Self { inner, optional }
    }
}

impl Formatter for NewtypeSchema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt_zod(f)?;
        if self.optional {
            f.write_str(".optional()")?;
        }
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt_ts(f)?;

        if self.optional {
            f.write_str(" | undefined")?;
        }
        Ok(())
    }
}
