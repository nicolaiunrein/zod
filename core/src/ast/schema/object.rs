use super::{Exported, NamedField, Ref};
use crate::ast::{Compiler, Delimited};

/// Representation of a `z.object({ ... })`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObjectSchema {
    fields: &'static [NamedField],
    extends: &'static [Ref],
}

impl ObjectSchema {
    pub const fn new(fields: &'static [NamedField]) -> Self {
        Self {
            fields,
            extends: &[],
        }
    }

    pub const fn with_extensions(self, extends: &'static [Ref]) -> Self {
        Self {
            fields: self.fields,
            extends,
        }
    }

    pub const fn export(self, name: &'static str) -> Exported<Self> {
        Exported::new(name, self)
    }

    pub const fn fields(&self) -> &'static [NamedField] {
        self.fields
    }

    pub fn generics(&self) -> impl Iterator<Item = &'static str> {
        self.fields.iter().filter_map(|f| f.value().get_generic())
    }

    pub fn is_generic(&self) -> bool {
        self.fields
            .iter()
            .any(|f| f.value().get_generic().is_some())
    }
}

impl Compiler for Exported<ObjectSchema> {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("const {} = ", self.name))?;
        if self.schema.is_generic() {
            f.write_str("(")?;

            self.schema
                .generics()
                .comma_separated(f, |f, g| f.write_fmt(format_args!("{g}: z.ZodTypeAny")))?;

            f.write_str(") => ")?;
            self.schema.fmt_zod(f)?;
        } else {
            f.write_str("z.lazy(() => ")?;
            self.schema.fmt_zod(f)?;
            f.write_str(")")?;
        }

        for ext in self.schema.extends {
            f.write_str(".extends(z.lazy(() => ")?;
            ext.fmt_zod(f)?;
            f.write_str("))")?;
        }

        f.write_str(";")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("interface ")?;
        f.write_str(self.name)?;

        if self.schema.is_generic() {
            f.write_str("<")?;
            self.schema
                .generics()
                .comma_separated(f, |f, g| f.write_str(g))?;
            f.write_str(">")?;
        }
        f.write_str(" ")?;

        if !self.schema.extends.is_empty() {
            f.write_str("extends ")?;
            self.schema
                .extends
                .iter()
                .comma_separated(f, |f, e| e.fmt_ts(f))?;
        }
        self.schema.fmt_ts(f)?;
        Ok(())
    }
}

impl Compiler for ObjectSchema {
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
    use crate::types::Usize;

    use super::*;

    const OBJECT: ObjectSchema = ObjectSchema::new(&[
        NamedField::new_req::<String>("a"),
        NamedField::new_req::<Usize>("b"),
    ]);

    const GENERIC: ObjectSchema = ObjectSchema::new(&[
        NamedField::generic("a", "A"),
        NamedField::new_req::<Usize>("b"),
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
            OBJECT.export("test").to_zod_string(),
            format!("const test = z.lazy(() => {});", OBJECT.to_zod_string()),
        );

        assert_eq!(
            OBJECT.export("test").to_ts_string(),
            format!("interface test {}", OBJECT.to_ts_string())
        );
    }

    #[test]
    fn generic_ok() {
        assert_eq!(GENERIC.to_zod_string(), "z.object({ a: A, b: Rs.Usize })");
        assert_eq!(GENERIC.to_ts_string(), "{ a: A, b: Rs.Usize }");

        assert_eq!(
            GENERIC.export("test").to_zod_string(),
            format!(
                "const test = (A: z.ZodTypeAny) => {};",
                GENERIC.to_zod_string()
            ),
        );

        assert_eq!(
            GENERIC.export("test").to_ts_string(),
            format!("interface test<A> {}", GENERIC.to_ts_string())
        );
    }
}
