use crate::ast::{Compiler, Delimited};

use super::{Exported, TupleField};

/// Representation of a `z.tuple([ ... ])`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TupleSchema {
    fields: &'static [TupleField],
}
impl TupleSchema {
    pub const fn new(fields: &'static [TupleField]) -> TupleSchema {
        Self { fields }
    }

    pub const fn export(self, name: &'static str) -> Exported<Self> {
        Exported::new(name, self)
    }
}

impl Compiler for Exported<TupleSchema> {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("const {} = z.lazy(() => ", self.name))?;
        f.write_str("z.tuple([")?;
        self.schema
            .fields
            .iter()
            .comma_separated(f, |f, field| field.fmt_zod(f))?;

        f.write_str("]));")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("type {} = [", self.name))?;
        self.schema
            .fields
            .iter()
            .comma_separated(f, |f, field| field.fmt_ts(f))?;
        f.write_str("];")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn tuple_ok() {
        const TUPLE: TupleSchema = TupleSchema::new(&[
            TupleField::new_req::<String>(),
            TupleField::new_req::<crate::types::Usize>(),
        ]);

        assert_eq!(
            TUPLE.export("test").to_zod_string(),
            format!("const test = z.lazy(() => z.tuple([Rs.String, Rs.Usize]));",)
        );
        assert_eq!(
            TUPLE.export("test").to_ts_string(),
            "type test = [Rs.String, Rs.Usize];"
        );
    }
}
