use crate::ast::{Compiler, Delimited};

use super::{Exported, Ref};

/// Representation of a `z.union([ ... ])`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnionSchema {
    variants: &'static [Ref],
}

impl UnionSchema {
    pub const fn new(variants: &'static [Ref]) -> UnionSchema {
        Self { variants }
    }

    pub const fn export(self, name: &'static str) -> Exported<Self> {
        Exported::new(name, self)
    }
}

impl Compiler for Exported<UnionSchema> {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("const {} = z.lazy(() => z.union([", self.name))?;
        self.schema
            .variants
            .iter()
            .comma_separated(f, |f, field| field.fmt_zod(f))?;

        f.write_str("]));")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("type {} = ", self.name))?;
        self.schema
            .variants
            .iter()
            .fmt_delimited(f, " | ", |f, field| field.fmt_ts(f))?;
        f.write_str(";")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn union_ok() {
        const DEF: UnionSchema = UnionSchema::new(&[
            Ref::new_req::<String>(),
            Ref::new_req::<crate::types::Usize>(),
        ]);

        assert_eq!(
            DEF.export("test").to_zod_string(),
            "const test = z.lazy(() => z.union([Rs.String, Rs.Usize]));"
        );
        assert_eq!(
            DEF.export("test").to_ts_string(),
            "type test = Rs.String | Rs.Usize;"
        );
    }
}
