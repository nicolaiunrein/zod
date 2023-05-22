use crate::ast::{Compiler, Delimited};

use super::{ExportSchema, Exported, NamedField};

/// Representation of a `z.discriminatedUnion("key", [ ... ])`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DiscriminatedUnionSchema {
    key: &'static str,
    variants: &'static [DiscriminatedVariant],
    generics: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DiscriminatedVariant {
    pub tag: &'static str,
    pub content_tag: Option<&'static str>,
    pub fields: &'static [NamedField],
}

impl DiscriminatedVariant {
    pub const fn raw_internally_tagged_newtype(tag: &'static str, schema: ExportSchema) -> Self {
        match schema {
            ExportSchema::Object(schema) => Self {
                tag,
                content_tag: None,
                fields: schema.fields(),
            },
            _ => panic!("expected ObjectSchema"),
        }
    }
}

impl DiscriminatedUnionSchema {
    pub const fn new(
        key: &'static str,
        variants: &'static [DiscriminatedVariant],
        generics: &'static [&'static str],
    ) -> DiscriminatedUnionSchema {
        Self {
            key,
            variants,
            generics,
        }
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

        self.schema.variants.iter().comma_separated(f, |f, v| {
            f.write_str("z.object({ ")?;

            f.write_fmt(format_args!(
                "{key}: z.literal(\"{value}\")",
                key = self.schema.key,
                value = v.tag
            ))?;

            if let Some(content) = v.content_tag {
                f.write_str(", ")?;
                f.write_fmt(format_args!("{content}: z.object({{ "))?;
                v.fields.iter().comma_separated(f, |f, field| {
                    field.resolve(self.schema.generics).fmt_zod(f)
                })?;
                f.write_str(" })")?;
            } else {
                if !v.fields.is_empty() {
                    f.write_str(", ")?;
                }

                v.fields.iter().comma_separated(f, |f, field| {
                    field.resolve(self.schema.generics).fmt_zod(f)
                })?;
            }

            f.write_str(" })")?;
            Ok(())
        })?;

        f.write_str("]));")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("type {name} = ", name = self.name))?;
        self.schema
            .variants
            .iter()
            .fmt_delimited(f, " | ", |f, v| {
                f.write_str("{ ")?;
                f.write_fmt(format_args!(
                    "{key}: \"{value}\"",
                    key = self.schema.key,
                    value = v.tag
                ))?;

                if let Some(content) = v.content_tag {
                    f.write_fmt(format_args!(", {content}: {{ "))?;
                    v.fields.iter().comma_separated(f, |f, field| {
                        field.resolve(self.schema.generics).fmt_ts(f)
                    })?;
                    f.write_str(" }")?;
                } else {
                    if !v.fields.is_empty() {
                        f.write_str(", ")?;
                    }

                    v.fields.iter().comma_separated(f, |f, field| {
                        field.resolve(self.schema.generics).fmt_ts(f)
                    })?;
                }

                f.write_str(" }")?;
                Ok(())
            })?;

        f.write_str(";")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::ast::{NamedField, Ref};
    use crate::types::{Isize, Usize};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn discriminated_union_ok() {
        const FIELDS: &[DiscriminatedVariant] = &[
            DiscriminatedVariant {
                tag: "A",
                content_tag: None,
                fields: &[NamedField::new("b", Ref::new_req::<Usize>())],
            },
            DiscriminatedVariant {
                tag: "B",
                content_tag: None,
                fields: &[NamedField::new("c", Ref::new_req::<Isize>())],
            },
        ];

        const DEF: DiscriminatedUnionSchema = DiscriminatedUnionSchema::new("myKey", FIELDS, &[]);
        assert_eq!(
            DEF.export("test").to_zod_string(),
            r#"const test = z.lazy(() => z.discriminatedUnion("myKey", [z.object({ myKey: z.literal("A"), b: Rs.Usize }), z.object({ myKey: z.literal("B"), c: Rs.Isize })]));"#
        );
        assert_eq!(
            DEF.export("test").to_ts_string(),
            r#"type test = { myKey: "A", b: Rs.Usize } | { myKey: "B", c: Rs.Isize };"#,
        );
    }

    #[test]
    fn discriminated_union_with_content_ok() {
        const FIELDS: &[DiscriminatedVariant] = &[
            DiscriminatedVariant {
                tag: "A",
                content_tag: Some("content"),
                fields: &[NamedField::new("b", Ref::new_req::<Usize>())],
            },
            DiscriminatedVariant {
                tag: "B",
                content_tag: Some("content"),
                fields: &[NamedField::new("c", Ref::new_req::<Isize>())],
            },
        ];

        const DEF: DiscriminatedUnionSchema = DiscriminatedUnionSchema::new("myKey", FIELDS, &[]);
        assert_eq!(
            DEF.export("test").to_zod_string(),
            r#"const test = z.lazy(() => z.discriminatedUnion("myKey", [z.object({ myKey: z.literal("A"), content: z.object({ b: Rs.Usize }) }), z.object({ myKey: z.literal("B"), content: z.object({ c: Rs.Isize }) })]));"#
        );
        assert_eq!(
            DEF.export("test").to_ts_string(),
            r#"type test = { myKey: "A", content: { b: Rs.Usize } } | { myKey: "B", content: { c: Rs.Isize } };"#,
        );
    }
}
