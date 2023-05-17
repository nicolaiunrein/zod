use crate::ast::Compiler;

use super::{Exported, Ref};

/// An type alias
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NewtypeSchema {
    inner: &'static Ref,
    optional: bool,
}
impl NewtypeSchema {
    pub const fn new(inner: &'static Ref, optional: bool) -> NewtypeSchema {
        Self { inner, optional }
    }

    pub const fn export(self, name: &'static str) -> Exported<Self> {
        Exported::new(name, self)
    }
}

impl Compiler for NewtypeSchema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt_zod(f)?;
        if self.optional {
            f.write_str(".optional()")?;
        }
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt_ts(f)?;

        if self.optional {
            f.write_str(" | undefined")?;
        }
        Ok(())
    }
}

impl Compiler for Exported<NewtypeSchema> {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // todo support generics
        f.write_fmt(format_args!("const {} = z.lazy(() => ", self.name))?;
        self.schema.fmt_zod(f)?;
        f.write_str(");")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // todo support generics
        f.write_fmt(format_args!("type {} = ", self.name))?;
        self.schema.fmt_ts(f)?;
        f.write_str(";")?;
        Ok(())
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn newtype_ok() {
        const NEWTYPE: NewtypeSchema =
            NewtypeSchema::new(&crate::ast::Ref::new_req::<String>(), false);

        assert_eq!(
            NEWTYPE.export("test").to_zod_string(),
            format!("const test = z.lazy(() => Rs.String);")
        );
        assert_eq!(
            NEWTYPE.export("test").to_ts_string(),
            "type test = Rs.String;",
        );
    }
}
