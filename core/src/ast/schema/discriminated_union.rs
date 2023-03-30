use crate::ast::{Compiler, Delimited};

use super::{Exported, ObjectSchema};

/// Representation of a `z.discriminatedUnion("key", [ ... ])`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DiscriminatedUnionSchema {
    key: &'static str,
    variants: &'static [ObjectSchema],
}
impl DiscriminatedUnionSchema {
    pub const fn new(
        key: &'static str,
        variants: &'static [ObjectSchema],
    ) -> DiscriminatedUnionSchema {
        Self { key, variants }
    }

    pub const fn export(self, name: &'static str) -> Exported<Self> {
        Exported::new(name, self)
    }
}

impl Compiler for Exported<DiscriminatedUnionSchema> {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "const {name} = z.lazy(() => z.discriminatedUnion(\"{key}\", [",
            name = self.name,
            key = self.schema.key
        ))?;

        self.schema
            .variants
            .iter()
            .comma_separated(f, |f, obj| obj.fmt_zod(f))?;

        f.write_str("]));")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("type {name} = ", name = self.name))?;
        self.schema
            .variants
            .iter()
            .fmt_delimited(f, " | ", |f, obj| obj.fmt_ts(f))?;

        f.write_str(";")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::ast::NamedField;
    use crate::types::{Isize, Usize};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn discriminated_union_ok() {
        const FIELDS: &[ObjectSchema] = &[
            ObjectSchema::new(&[
                NamedField::new_req::<String>("myKey"),
                NamedField::new_req::<Usize>("b"),
            ]),
            ObjectSchema::new(&[
                NamedField::new_req::<String>("myKey"),
                NamedField::new_req::<Isize>("c"),
            ]),
        ];

        const DEF: DiscriminatedUnionSchema = DiscriminatedUnionSchema::new("myKey", FIELDS);
        assert_eq!(
            DEF.export("test").to_zod_string(),
            format!(
                "const test = z.lazy(() => z.discriminatedUnion(\"myKey\", [{}, {}]));",
                FIELDS[0].to_zod_string(),
                FIELDS[1].to_zod_string()
            )
        );
        assert_eq!(
            DEF.export("test").to_ts_string(),
            format!(
                "type test = {} | {};",
                FIELDS[0].to_ts_string(),
                FIELDS[1].to_ts_string()
            )
        );
    }
}
