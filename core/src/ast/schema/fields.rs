use super::{Compiler, OwnedRef, Ref};

/// A name/value pair as used in objects
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamedField {
    name: &'static str,
    value: Ref,
    optional: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnedNamedField {
    name: &'static str,
    value: OwnedRef,
    optional: bool,
}

impl NamedField {
    pub const fn new(name: &'static str, value: Ref) -> Self {
        Self {
            name,
            value,
            optional: false,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn value(&self) -> Ref {
        self.value
    }

    pub const fn optional(self) -> Self {
        Self {
            optional: true,
            ..self
        }
    }

    pub(crate) fn transform(&self, generics: &[&'static str]) -> OwnedNamedField {
        OwnedNamedField {
            name: self.name,
            optional: self.optional,
            value: self.value.transform(generics),
        }
    }
}

impl Compiler for OwnedNamedField {
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
    value: Ref,
    optional: bool,
}

/// A name/value pair as used in objects
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OwnedTupleField {
    value: OwnedRef,
    optional: bool,
}

impl TupleField {
    pub const fn new(value: Ref) -> Self {
        Self {
            value,
            optional: false,
        }
    }

    pub const fn value(&self) -> Ref {
        self.value
    }

    pub const fn optional(self) -> Self {
        Self {
            optional: true,
            ..self
        }
    }

    pub fn transform(&self, generics: &[&'static str]) -> OwnedTupleField {
        OwnedTupleField {
            optional: self.optional,
            value: self.value.transform(generics),
        }
    }
}

impl Compiler for OwnedTupleField {
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
        const FIELD: NamedField = NamedField::new("test", Ref::new_req::<Usize>());
        assert_eq!(FIELD.transform(&[]).to_zod_string(), "test: Rs.Usize");
        assert_eq!(FIELD.transform(&[]).to_ts_string(), "test: Rs.Usize");
    }

    #[test]
    fn named_field_optional() {
        const FIELD: NamedField = NamedField::new("test", Ref::new_req::<Usize>()).optional();

        assert_eq!(
            FIELD.transform(&[]).to_zod_string(),
            "test: Rs.Usize.optional()"
        );
        assert_eq!(
            FIELD.transform(&[]).to_ts_string(),
            "test?: Rs.Usize | undefined"
        );
    }

    #[test]
    fn tuple_field_non_optional() {
        const FIELD: TupleField = TupleField::new(Ref::new_req::<crate::types::Usize>());
        assert_eq!(FIELD.transform(&[]).to_zod_string(), "Rs.Usize");
        assert_eq!(FIELD.transform(&[]).to_ts_string(), "Rs.Usize");
    }

    #[test]
    fn tuple_field_optional() {
        const FIELD: TupleField = TupleField::new(Ref::new_req::<crate::types::Usize>()).optional();

        assert_eq!(FIELD.transform(&[]).to_zod_string(), "Rs.Usize.optional()");
        assert_eq!(FIELD.transform(&[]).to_ts_string(), "Rs.Usize | undefined");
    }
}
