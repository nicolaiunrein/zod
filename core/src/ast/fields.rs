use super::{FormatInlinedTs, FormatInlinedZod, FormatTypescript, FormatZod, ZodDefinition};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StructFields {
    Named(&'static [MaybeFlatField]),
    Tuple(&'static [TupleField]),
    Transparent {
        value: &'static ZodDefinition,
        optional: bool,
    },
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
    pub value: &'static ZodDefinition,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TupleField {
    pub optional: bool,
    pub value: &'static ZodDefinition,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamedField {
    pub optional: bool,
    pub name: &'static str,
    pub value: &'static ZodDefinition,
}

impl MaybeFlatField {
    pub const fn new_flat(value: &'static ZodDefinition) -> Self {
        Self::Flat(FlatField { value })
    }
}

impl FormatZod for NamedField {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)?;
        f.write_str(": ")?;
        self.value.ty().fmt_inlined_zod(f)?;
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
        self.value.ty().fmt_inlined_ts(f)?;
        if self.optional {
            f.write_str(" | undefined")?;
        }
        Ok(())
    }
}

impl FormatZod for TupleField {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.ty().fmt_inlined_zod(f)?;
        if self.optional {
            f.write_str(".optional()")?;
        }
        Ok(())
    }
}

impl FormatTypescript for TupleField {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.ty().fmt_inlined_ts(f)?;
        if self.optional {
            f.write_str(" | undefined")?;
        }
        Ok(())
    }
}

impl FormatZod for FlatField {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(".extend(z.lazy(() => ")?;
        self.value.ty().fmt_inlined_zod(f)?;
        f.write_str("))")?;
        Ok(())
    }
}

impl FormatTypescript for FlatField {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.ty().fmt_inlined_ts(f)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::ZodType;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn zod_inner_tuple_struct_field() {
        assert_eq!(
            TupleField {
                optional: false,
                value: &String::AST.def,
            }
            .to_zod_string(),
            "Rs.String"
        );
    }

    #[test]
    fn zod_inner_tuple_struct_field_optional() {
        let field = TupleField {
            optional: true,
            value: &String::AST.def,
        };
        assert_eq!(field.to_zod_string(), "Rs.String.optional()");
        assert_eq!(field.to_ts_string(), "Rs.String | undefined");
    }

    #[test]
    fn zod_named_struct_field() {
        let field = NamedField {
            optional: false,
            name: "my_value",
            value: &String::AST.def,
        };
        assert_eq!(field.to_zod_string(), "my_value: Rs.String");
        assert_eq!(field.to_ts_string(), "my_value: Rs.String");
    }

    #[test]
    fn zod_named_struct_field_optional() {
        let field = NamedField {
            optional: true,
            name: "my_value",
            value: &<()>::AST.def,
        };
        assert_eq!(field.to_zod_string(), "my_value: Rs.Unit.optional()");
        assert_eq!(field.to_ts_string(), "my_value?: Rs.Unit | undefined");
    }

    #[test]
    fn flattened_field() {
        let field = FlatField {
            value: &<Option<()>>::AST.def,
        };
        assert_eq!(
            field.to_zod_string(),
            ".extend(z.lazy(() => Rs.Option(Rs.Unit)))"
        );
        assert_eq!(field.to_ts_string(), "Rs.Option<Rs.Unit>");
    }
}
