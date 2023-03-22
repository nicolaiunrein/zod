use super::{Formatter, InlineSchema, Node};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamedField {
    name: &'static str,
    value: InlineSchema,
}

impl NamedField {
    pub fn new<T: Node>(name: &'static str) -> Self {
        Self {
            name,
            value: T::inline(),
        }
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
