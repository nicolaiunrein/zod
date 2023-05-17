use crate::ast::Compiler;

use super::{Exported, TupleField};

/// An type alias
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NewtypeSchema {
    inner: &'static TupleField,
}
impl NewtypeSchema {
    pub const fn new(inner: &'static TupleField) -> NewtypeSchema {
        Self { inner }
    }

    pub const fn export(self, name: &'static str) -> Exported<Self> {
        Exported::new(name, self)
    }
}

impl Compiler for NewtypeSchema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt_zod(f)?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt_ts(f)?;
        Ok(())
    }
}

impl Compiler for Exported<NewtypeSchema> {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.schema.inner.value().get_generic() {
            Some(value) => {
                f.write_fmt(format_args!(
                    "const {} = ({value}: z.ZodTypeAny) => z.lazy(() => ",
                    self.name
                ))?;
            }
            None => {
                f.write_fmt(format_args!("const {} = z.lazy(() => ", self.name))?;
            }
        }

        self.schema.fmt_zod(f)?;
        f.write_str(");")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.schema.inner.value().get_generic() {
            Some(value) => {
                f.write_fmt(format_args!("type {}<{value}> = ", self.name))?;
            }
            None => {
                f.write_fmt(format_args!("type {} = ", self.name))?;
            }
        }
        self.schema.fmt_ts(f)?;
        f.write_str(";")?;
        Ok(())
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn newtype_ok() {
        const NEWTYPE: NewtypeSchema = NewtypeSchema::new(&TupleField::new_req::<String>());

        assert_eq!(
            NEWTYPE.export("test").to_zod_string(),
            format!("const test = z.lazy(() => Rs.String);")
        );
        assert_eq!(
            NEWTYPE.export("test").to_ts_string(),
            "type test = Rs.String;",
        );
    }

    #[test]
    fn newtype_generic_ok() {
        const NEWTYPE: NewtypeSchema = NewtypeSchema::new(&TupleField::generic("T"));

        assert_eq!(
            NEWTYPE.export("test").to_zod_string(),
            format!("const test = (T: z.ZodTypeAny) => z.lazy(() => T);")
        );

        assert_eq!(NEWTYPE.export("test").to_ts_string(), "type test<T> = T;");
    }
}
