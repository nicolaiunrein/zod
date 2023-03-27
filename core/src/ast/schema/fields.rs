use crate::RequestType;

use super::{Formatter, InlineSchema};

/// A name/value pair as used in objects
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamedField {
    name: &'static str,
    value: InlineSchema,
    optional: bool,
}

impl NamedField {
    pub const fn new<T: RequestType>(name: &'static str) -> Self {
        Self {
            name,
            value: T::AST.inline(),
            optional: false,
        }
    }
    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn value(&self) -> InlineSchema {
        self.value
    }

    pub const fn optional(self) -> Self {
        Self {
            optional: true,
            ..self
        }
    }
}

impl Formatter for NamedField {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)?;
        f.write_str(": ")?;
        self.value.fmt_zod(f)?;
        if self.optional {
            f.write_str(".optional()")?;
        }
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)?;
        if self.optional {
            f.write_str("?")?;
        }
        f.write_str(": ")?;
        self.value.fmt_ts(f)?;
        if self.optional {
            f.write_str(" | undefined")?;
        }
        Ok(())
    }
}

/// A name/value pair as used in objects
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TupleField {
    value: InlineSchema,
    optional: bool,
}

impl TupleField {
    pub const fn new<T: RequestType>() -> Self {
        Self {
            value: T::AST.inline(),
            optional: false,
        }
    }

    pub const fn value(&self) -> InlineSchema {
        self.value
    }

    pub const fn optional(self) -> Self {
        Self {
            optional: true,
            ..self
        }
    }
}

impl Formatter for TupleField {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt_zod(f)?;
        if self.optional {
            f.write_str(".optional()")?;
        }
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt_ts(f)?;
        if self.optional {
            f.write_str(" | undefined")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn named_field_non_optional() {
        const FIELD: NamedField = NamedField::new::<crate::types::Usize>("test");
        assert_eq!(FIELD.to_zod_string(), "test: Rs.Usize");
        assert_eq!(FIELD.to_ts_string(), "test: Rs.Usize");
    }

    #[test]
    fn named_field_optional() {
        const FIELD: NamedField = NamedField::new::<crate::types::Usize>("test").optional();

        assert_eq!(FIELD.to_zod_string(), "test: Rs.Usize.optional()");
        assert_eq!(FIELD.to_ts_string(), "test?: Rs.Usize | undefined");
    }

    #[test]
    fn tuple_field_non_optional() {
        const FIELD: TupleField = TupleField::new::<crate::types::Usize>();
        assert_eq!(FIELD.to_zod_string(), "Rs.Usize");
        assert_eq!(FIELD.to_ts_string(), "Rs.Usize");
    }

    #[test]
    fn tuple_field_optional() {
        const FIELD: TupleField = TupleField::new::<crate::types::Usize>().optional();

        assert_eq!(FIELD.to_zod_string(), "Rs.Usize.optional()");
        assert_eq!(FIELD.to_ts_string(), "Rs.Usize | undefined");
    }
}
