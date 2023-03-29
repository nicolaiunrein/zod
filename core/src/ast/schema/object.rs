use super::NamedField;
use crate::ast::{Delimited, Formatter};

/// Representation of a `z.object({ ... })`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObjectSchema {
    fields: &'static [NamedField],
}

impl ObjectSchema {
    pub const fn new(fields: &'static [NamedField]) -> Self {
        Self { fields }
    }

    pub fn generics(&self) -> impl Iterator<Item = &'static str> {
        self.fields.iter().filter_map(|f| f.value().get_generic())
    }

    pub fn is_generic(&self) -> bool {
        self.fields
            .iter()
            .find(|f| f.value().get_generic().is_some())
            .is_some()
    }

    pub const fn fields(&self) -> &'static [NamedField] {
        self.fields
    }
}

impl Formatter for (&'static str, ObjectSchema) {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.0;
        let schema = self.1;
        f.write_fmt(format_args!("const {name} = "))?;
        if schema.is_generic() {
            f.write_str("(")?;

            schema
                .generics()
                .comma_separated(f, |f, g| f.write_fmt(format_args!("{g}: z.ZodTypeAny")))?;

            f.write_str(") => ")?;
            schema.fmt_zod(f)?;
        } else {
            f.write_str("z.lazy(() => ")?;
            schema.fmt_zod(f)?;
            f.write_str(")")?;
        }
        f.write_str(";")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.0;
        let schema = self.1;

        f.write_str("interface ")?;
        f.write_str(name)?;

        // todo
        let mut generics = schema.generics().peekable();
        if generics.peek().is_some() {
            f.write_str("<")?;
            while let Some(gen) = generics.next() {
                f.write_str(gen)?;
                if generics.peek().is_some() {
                    f.write_str(", ")?;
                }
            }
            f.write_str(">")?;
        }
        f.write_str(" ")?;
        schema.fmt_ts(f)?;
        Ok(())
    }
}

impl Formatter for ObjectSchema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.object({ ")?;
        self.fields
            .iter()
            .comma_separated(f, |f, field| field.fmt_zod(f))?;

        f.write_str(" })")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("{ ")?;

        self.fields
            .iter()
            .comma_separated(f, |f, field| field.fmt_ts(f))?;
        f.write_str(" }")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Ref;
    use crate::types::Usize;

    use super::*;

    const OBJECT: ObjectSchema = ObjectSchema::new(&[
        NamedField::new("a", Ref::new_req::<String>()),
        NamedField::new("b", Ref::new_req::<Usize>()),
    ]);

    #[test]
    fn object_ok() {
        assert_eq!(
            OBJECT.to_zod_string(),
            "z.object({ a: Rs.String, b: Rs.Usize })"
        );

        assert_eq!(OBJECT.to_ts_string(), "{ a: Rs.String, b: Rs.Usize }")
    }

    #[test]
    fn object_export_ok() {
        assert_eq!(
            ("test", OBJECT).to_zod_string(),
            format!("const test = z.lazy(() => {});", OBJECT.to_zod_string()),
        );

        assert_eq!(
            ("test", OBJECT).to_ts_string(),
            format!("interface test {}", OBJECT.to_ts_string())
        );
    }
}
