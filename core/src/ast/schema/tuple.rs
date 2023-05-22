use crate::ast::{Compiler, Delimited};

use super::{Exported, TupleField};

/// Representation of a `z.tuple([ ... ])`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TupleSchema {
    fields: &'static [TupleField],
    generics: &'static [&'static str],
}
impl TupleSchema {
    pub const fn new(
        fields: &'static [TupleField],
        generics: &'static [&'static str],
    ) -> TupleSchema {
        Self { fields, generics }
    }

    pub const fn export(self, name: &'static str) -> Exported<Self> {
        Exported::new(name, self)
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

impl Compiler for TupleSchema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.tuple([")?;
        self.fields
            .iter()
            .map(|f| f.resolve(self.generics))
            .comma_separated(f, |f, field| field.fmt_zod(f))?;

        f.write_str("])")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[")?;
        self.fields
            .iter()
            .map(|f| f.resolve(self.generics))
            .comma_separated(f, |f, field| field.fmt_ts(f))?;
        f.write_str("]")?;
        Ok(())
    }
}

impl Compiler for Exported<TupleSchema> {
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

        f.write_str(";")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("type {}", self.name))?;

        if self.schema.is_generic() {
            f.write_str("<")?;
            self.schema
                .generics()
                .comma_separated(f, |f, g| f.write_str(g))?;
            f.write_str(">")?;
        }

        f.write_str(" = ")?;

        self.schema.fmt_ts(f)?;
        f.write_str(";")?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Ref;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn tuple_ok() {
        const TUPLE: TupleSchema = TupleSchema::new(
            &[
                TupleField::new(Ref::new_req::<String>()),
                TupleField::new(Ref::new_req::<crate::types::Usize>()),
            ],
            &[],
        );

        assert_eq!(
            TUPLE.export("test").to_zod_string(),
            format!("const test = z.lazy(() => z.tuple([Rs.String, Rs.Usize]));",)
        );
        assert_eq!(
            TUPLE.export("test").to_ts_string(),
            "type test = [Rs.String, Rs.Usize];"
        );
    }

    #[test]
    fn tuple_generic_ok() {
        const TUPLE: TupleSchema = TupleSchema::new(
            &[
                TupleField::new(Ref::generic("T")),
                TupleField::new(Ref::new_req::<crate::types::Usize>()),
            ],
            &[],
        );

        assert_eq!(
            TUPLE.export("test").to_zod_string(),
            format!("const test = (T: z.ZodTypeAny) => z.tuple([T, Rs.Usize]);",)
        );
        assert_eq!(
            TUPLE.export("test").to_ts_string(),
            "type test<T> = [T, Rs.Usize];"
        );
    }
}
