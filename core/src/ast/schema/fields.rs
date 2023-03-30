use crate::{RequestType, ResponseType};

use super::{Formatter, Ref};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FieldValue {
    Resolved(Ref),
    Generic(&'static str),
}

impl FieldValue {
    pub const fn is_generic(&self) -> bool {
        match self {
            FieldValue::Resolved(_) => false,
            FieldValue::Generic(_) => true,
        }
    }

    pub(crate) fn get_generic(&self) -> Option<&'static str> {
        match self {
            FieldValue::Resolved(_) => None,
            FieldValue::Generic(value) => Some(value),
        }
    }
}

impl Formatter for FieldValue {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldValue::Resolved(value) => value.fmt_zod(f),
            FieldValue::Generic(value) => f.write_str(value),
        }
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldValue::Resolved(value) => value.fmt_ts(f),
            FieldValue::Generic(value) => f.write_str(value),
        }
    }
}

/// A name/value pair as used in objects
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamedField {
    name: &'static str,
    value: FieldValue,
    optional: bool,
}

impl NamedField {
    pub const fn new_req<T: RequestType>(name: &'static str) -> Self {
        Self {
            name,
            value: FieldValue::Resolved(Ref::new_req::<T>()),
            optional: false,
        }
    }

    pub const fn new_res<T: ResponseType>(name: &'static str) -> Self {
        Self {
            name,
            value: FieldValue::Resolved(Ref::new_res::<T>()),
            optional: false,
        }
    }

    pub const fn generic(name: &'static str, value: &'static str) -> Self {
        Self {
            name,
            value: FieldValue::Generic(value),
            optional: false,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn value(&self) -> FieldValue {
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
    value: FieldValue,
    optional: bool,
}

impl TupleField {
    pub const fn new_req<T: RequestType>() -> Self {
        Self {
            value: FieldValue::Resolved(Ref::new_req::<T>()),
            optional: false,
        }
    }

    pub const fn new_res<T: ResponseType>() -> Self {
        Self {
            value: FieldValue::Resolved(Ref::new_res::<T>()),
            optional: false,
        }
    }

    pub const fn value(&self) -> FieldValue {
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
    use crate::types::Usize;

    use super::*;

    #[test]
    fn named_field_non_optional() {
        const FIELD: NamedField = NamedField::new_req::<Usize>("test");
        assert_eq!(FIELD.to_zod_string(), "test: Rs.Usize");
        assert_eq!(FIELD.to_ts_string(), "test: Rs.Usize");
    }

    #[test]
    fn named_field_optional() {
        const FIELD: NamedField = NamedField::new_req::<Usize>("test").optional();

        assert_eq!(FIELD.to_zod_string(), "test: Rs.Usize.optional()");
        assert_eq!(FIELD.to_ts_string(), "test?: Rs.Usize | undefined");
    }

    #[test]
    fn tuple_field_non_optional() {
        const FIELD: TupleField = TupleField::new_req::<crate::types::Usize>();
        assert_eq!(FIELD.to_zod_string(), "Rs.Usize");
        assert_eq!(FIELD.to_ts_string(), "Rs.Usize");
    }

    #[test]
    fn tuple_field_optional() {
        const FIELD: TupleField = TupleField::new_req::<crate::types::Usize>().optional();

        assert_eq!(FIELD.to_zod_string(), "Rs.Usize.optional()");
        assert_eq!(FIELD.to_ts_string(), "Rs.Usize | undefined");
    }
}
