use crate::ast::{Compiler, Delimited};

use super::{ExportSchema, Exported, NamedField};

/// Representation of a `z.discriminatedUnion("key", [ ... ])`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DiscriminatedUnionSchema {
    key: &'static str,
    variants: &'static [DiscriminatedVariant],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DiscriminatedVariant {
    InternallyTagged(&'static str, &'static [NamedField]),
}

impl DiscriminatedVariant {
    pub const fn raw_internally_tagged_newtype(tag: &'static str, schema: ExportSchema) -> Self {
        match schema {
            ExportSchema::Object(schema) => Self::InternallyTagged(tag, schema.fields()),
            _ => panic!("expected ObjectSchema"),
        }
    }
}

impl DiscriminatedUnionSchema {
    pub const fn new(
        key: &'static str,
        variants: &'static [DiscriminatedVariant],
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
            .comma_separated(f, |f, v| match v {
                DiscriminatedVariant::InternallyTagged(tag_value, fields) => {
                    f.write_str("z.object({ ")?;

                    f.write_fmt(format_args!(
                        "{key}: z.literal(\"{value}\")",
                        key = self.schema.key,
                        value = tag_value
                    ))?;

                    if !fields.is_empty() {
                        f.write_str(", ")?;
                    }

                    fields
                        .iter()
                        .comma_separated(f, |f, field| field.fmt_zod(f))?;

                    f.write_str(" })")?;
                    Ok(())
                }
            })?;

        f.write_str("]));")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("type {name} = ", name = self.name))?;
        self.schema
            .variants
            .iter()
            .fmt_delimited(f, " | ", |f, v| match v {
                DiscriminatedVariant::InternallyTagged(tag_value, fields) => {
                    f.write_str("{ ")?;
                    f.write_fmt(format_args!(
                        "{key}: \"{value}\"",
                        key = self.schema.key,
                        value = tag_value
                    ))?;

                    if !fields.is_empty() {
                        f.write_str(", ")?;
                    }

                    fields
                        .iter()
                        .comma_separated(f, |f, field| field.fmt_ts(f))?;

                    f.write_str(" }")?;
                    Ok(())
                }
            })?;

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
        const FIELDS: &[DiscriminatedVariant] = &[
            DiscriminatedVariant::InternallyTagged("A", &[NamedField::new_req::<Usize>("b")]),
            DiscriminatedVariant::InternallyTagged("B", &[NamedField::new_req::<Isize>("c")]),
        ];

        const DEF: DiscriminatedUnionSchema = DiscriminatedUnionSchema::new("myKey", FIELDS);
        assert_eq!(
            DEF.export("test").to_zod_string(),
            r#"const test = z.lazy(() => z.discriminatedUnion("myKey", [z.object({ myKey: z.literal("A"), b: Rs.Usize }), z.object({ myKey: z.literal("B"), c: Rs.Isize })]));"#
        );
        assert_eq!(
            DEF.export("test").to_ts_string(),
            r#"type test = { myKey: "A", b: Rs.Usize } | { myKey: "B", c: Rs.Isize };"#,
        );
    }
}
