use crate::Node;

use super::{Formatter, InlineSchema};

/// A name/value pair as used in objects
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamedField {
    name: &'static str,
    value: InlineSchema,
}

impl NamedField {
    pub const fn new<T: Node>(name: &'static str) -> Self {
        Self {
            name,
            value: T::AST.inline(),
        }
    }
    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn value(&self) -> InlineSchema {
        self.value
    }
}

impl Formatter for NamedField {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)?;
        f.write_str(": ")?;
        self.value.fmt_zod(f)
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)?;
        f.write_str(": ")?;
        self.value.fmt_ts(f)
    }
}
