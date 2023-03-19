use crate::Inlined;

use super::{FormatTypescript, FormatZod, Generic, QualifiedType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StructFields {
    Named(&'static [MaybeFlatField]),
    Tuple(&'static [TupleField]),
    Transparent { value: FieldValue, optional: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FieldValue {
    Generic(Generic),
    Qualified(QualifiedType),
    Inlined(Inlined),
}

impl FormatZod for FieldValue {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldValue::Generic(inner) => inner.fmt_zod(f),
            FieldValue::Qualified(inner) => inner.fmt_zod(f),
            FieldValue::Inlined(inner) => <Inlined as std::fmt::Display>::fmt(inner, f),
        }
    }
}

impl FormatTypescript for FieldValue {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldValue::Generic(inner) => inner.fmt_ts(f),
            FieldValue::Qualified(inner) => inner.fmt_ts(f),
            FieldValue::Inlined(inner) => <Inlined as std::fmt::Display>::fmt(inner, f),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MaybeFlatField {
    Flat(FlatField),
    Named(NamedField),
}

impl MaybeFlatField {
    pub fn partition(fields: &[Self]) -> (Vec<NamedField>, Vec<FlatField>) {
        let mut inner = Vec::new();
        let mut flat = Vec::new();

        for field in fields.into_iter() {
            match field {
                Self::Flat(f) => flat.push(f.clone()),
                Self::Named(f) => inner.push(f.clone()),
            }
        }

        (inner, flat)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FlatField {
    // TODO: find a way to express flat optional fields in typescript with interfaces
    // see: https://github.com/nicolaiunrein/zod/issues/3
    pub value: FieldValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TupleField {
    pub optional: bool,
    pub value: FieldValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamedField {
    pub optional: bool,
    pub name: &'static str,
    pub value: FieldValue,
}

impl MaybeFlatField {
    pub const fn new_flat(value: FieldValue) -> Self {
        Self::Flat(FlatField { value })
    }
}

impl FormatZod for NamedField {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)?;
        f.write_str(": ")?;
        self.value.fmt_zod(f)?;
        if self.optional {
            f.write_str(".optional()")?;
        }
        Ok(())
    }
}

impl FormatTypescript for NamedField {
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

impl FormatZod for TupleField {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt_zod(f)?;
        if self.optional {
            f.write_str(".optional()")?;
        }
        Ok(())
    }
}

impl FormatTypescript for TupleField {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt_ts(f)?;
        if self.optional {
            f.write_str(" | undefined")?;
        }
        Ok(())
    }
}

impl FormatZod for FlatField {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(".extend(z.lazy(() => ")?;
        self.value.fmt_zod(f)?;
        f.write_str("))")?;
        Ok(())
    }
}

impl FormatTypescript for FlatField {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt_zod(f)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn zod_inner_tuple_struct_field() {
        assert_eq!(
            TupleField {
                optional: false,
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Ns",
                    ident: "myValue",
                    generics: Default::default()
                })
            }
            .to_zod_string(),
            "Ns.myValue"
        );
    }

    #[test]
    fn zod_inner_tuple_struct_field_optional() {
        let field = TupleField {
            optional: true,
            value: FieldValue::Qualified(QualifiedType {
                ns: "Ns",
                ident: "myValue",
                generics: Default::default(),
            }),
        };
        assert_eq!(field.to_zod_string(), "Ns.myValue.optional()");
        assert_eq!(field.to_ts_string(), "Ns.myValue | undefined");
    }

    #[test]
    fn zod_named_struct_field() {
        let field = NamedField {
            optional: false,
            name: "my_value",
            value: FieldValue::Qualified(QualifiedType {
                ns: "Ns",
                ident: "myValue",
                generics: Default::default(),
            }),
        };
        assert_eq!(field.to_zod_string(), "my_value: Ns.myValue");
        assert_eq!(field.to_ts_string(), "my_value: Ns.myValue");
    }

    #[test]
    fn zod_named_struct_field_optional() {
        let field = NamedField {
            optional: true,
            name: "my_value",
            value: FieldValue::Qualified(QualifiedType {
                ns: "Ns",
                ident: "myValue",
                generics: Default::default(),
            }),
        };
        assert_eq!(field.to_zod_string(), "my_value: Ns.myValue.optional()");
        assert_eq!(field.to_ts_string(), "my_value?: Ns.myValue | undefined");
    }

    #[test]
    fn flattened_field() {
        let field = FlatField {
            value: FieldValue::Qualified(QualifiedType {
                ns: "Ns",
                ident: "myValue",
                generics: Default::default(),
            }),
        };
        assert_eq!(field.to_zod_string(), ".extend(z.lazy(() => Ns.myValue))");
        assert_eq!(field.to_ts_string(), "Ns.myValue");
    }
}
