use crate::ast::{Compiler, Delimited};

use super::{Exported, NewtypeSchema, ObjectSchema, TupleSchema};

/// Representation of a `z.union([ ... ])`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnionSchema {
    variants: &'static [Variant],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VariantValue {
    Object(ObjectSchema),
    Tuple(TupleSchema),
    Newtype(NewtypeSchema),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Variant {
    ExternallyTagged(&'static str, Option<VariantValue>),
}

impl Compiler for VariantValue {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariantValue::Object(inner) => inner.fmt_zod(f),
            VariantValue::Tuple(inner) => inner.fmt_zod(f),
            VariantValue::Newtype(inner) => inner.fmt_zod(f),
        }
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariantValue::Object(inner) => inner.fmt_ts(f),
            VariantValue::Tuple(inner) => inner.fmt_ts(f),
            VariantValue::Newtype(inner) => inner.fmt_ts(f),
        }
    }
}

impl Compiler for Variant {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variant::ExternallyTagged(tag, value) => match value {
                None => f.write_fmt(format_args!("z.literal(\"{tag}\")")),
                Some(value) => {
                    f.write_fmt(format_args!("z.object({{ {tag}: "))?;
                    value.fmt_zod(f)?;
                    f.write_str(" })")?;
                    Ok(())
                }
            },
        }
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Variant::ExternallyTagged(tag, value) => match value {
                None => f.write_fmt(format_args!("\"{tag}\"")),
                Some(value) => {
                    f.write_fmt(format_args!("{{ {tag}: "))?;
                    value.fmt_ts(f)?;
                    f.write_str(" }")?;
                    Ok(())
                }
            },
        }
    }
}

impl UnionSchema {
    pub const fn new(variants: &'static [Variant]) -> UnionSchema {
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
        if self.schema.variants.is_empty() {
            f.write_str("never")?;
        } else {
            self.schema
                .variants
                .iter()
                .fmt_delimited(f, " | ", |f, field| field.fmt_ts(f))?;
        }
        f.write_str(";")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ast::{NamedField, Ref, TupleField};
    use crate::types::Usize;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn union_object_ok() {
        const DEF: UnionSchema = UnionSchema::new(&[
            Variant::ExternallyTagged(
                "A",
                Some(VariantValue::Object(ObjectSchema::new(&[
                    NamedField::new_req::<String>("a"),
                ]))),
            ),
            Variant::ExternallyTagged(
                "B",
                Some(VariantValue::Object(ObjectSchema::new(&[
                    NamedField::new_req::<Usize>("b"),
                ]))),
            ),
        ]);

        assert_eq!(
            DEF.export("test").to_zod_string(),
            "const test = z.lazy(() => z.union([z.object({ A: z.object({ a: Rs.String }) }), z.object({ B: z.object({ b: Rs.Usize }) })]));"
        );
        assert_eq!(
            DEF.export("test").to_ts_string(),
            "type test = { A: { a: Rs.String } } | { B: { b: Rs.Usize } };"
        );
    }

    #[test]
    fn union_tuple_ok() {
        const DEF: UnionSchema = UnionSchema::new(&[
            Variant::ExternallyTagged(
                "A",
                Some(VariantValue::Tuple(TupleSchema::new(&[
                    TupleField::new_req::<String>(),
                    TupleField::new_req::<()>(),
                ]))),
            ),
            Variant::ExternallyTagged(
                "B",
                Some(VariantValue::Tuple(TupleSchema::new(&[
                    TupleField::new_req::<Usize>(),
                    TupleField::new_req::<bool>(),
                ]))),
            ),
        ]);

        assert_eq!(
            DEF.export("test").to_zod_string(),
            "const test = z.lazy(() => z.union([z.object({ A: z.tuple([Rs.String, Rs.Unit]) }), z.object({ B: z.tuple([Rs.Usize, Rs.Bool]) })]));"
        );
        assert_eq!(
            DEF.export("test").to_ts_string(),
            "type test = { A: [Rs.String, Rs.Unit] } | { B: [Rs.Usize, Rs.Bool] };"
        );
    }

    #[test]
    fn union_newtype_ok() {
        const DEF: UnionSchema = UnionSchema::new(&[
            Variant::ExternallyTagged(
                "A",
                Some(VariantValue::Newtype(NewtypeSchema::new(
                    &Ref::new_req::<String>(),
                    false,
                ))),
            ),
            Variant::ExternallyTagged(
                "B",
                Some(VariantValue::Newtype(NewtypeSchema::new(
                    &Ref::new_req::<Usize>(),
                    true,
                ))),
            ),
        ]);

        assert_eq!(
            DEF.export("test").to_zod_string(),
            "const test = z.lazy(() => z.union([z.object({ A: Rs.String }), z.object({ B: Rs.Usize.optional() })]));"
        );
        assert_eq!(
            DEF.export("test").to_ts_string(),
            "type test = { A: Rs.String } | { B: Rs.Usize | undefined };"
        );
    }

    #[test]
    fn union_unit() {
        const DEF: UnionSchema = UnionSchema::new(&[
            Variant::ExternallyTagged("A", None),
            Variant::ExternallyTagged("B", None),
        ]);

        assert_eq!(
            DEF.export("test").to_zod_string(),
            "const test = z.lazy(() => z.union([z.literal(\"A\"), z.literal(\"B\")]));"
        );
        assert_eq!(
            DEF.export("test").to_ts_string(),
            "type test = \"A\" | \"B\";"
        );
    }

    #[test]
    fn union_mixed_ok() {
        const DEF: UnionSchema = UnionSchema::new(&[
            Variant::ExternallyTagged(
                "A",
                Some(VariantValue::Newtype(NewtypeSchema::new(
                    &Ref::new_req::<String>(),
                    true,
                ))),
            ),
            Variant::ExternallyTagged(
                "B",
                Some(VariantValue::Tuple(TupleSchema::new(&[
                    TupleField::new_req::<String>(),
                    TupleField::new_req::<()>(),
                ]))),
            ),
            Variant::ExternallyTagged(
                "C",
                Some(VariantValue::Object(ObjectSchema::new(&[
                    NamedField::new_req::<Usize>("b"),
                ]))),
            ),
        ]);

        assert_eq!(
            DEF.export("test").to_zod_string(),
            "const test = z.lazy(() => z.union([z.object({ A: Rs.String.optional() }), z.object({ B: z.tuple([Rs.String, Rs.Unit]) }), z.object({ C: z.object({ b: Rs.Usize }) })]));"
        );
        assert_eq!(
            DEF.export("test").to_ts_string(),
            "type test = { A: Rs.String | undefined } | { B: [Rs.String, Rs.Unit] } | { C: { b: Rs.Usize } };"
        );
    }

    #[test]
    fn empty_union() {
        const DEF: UnionSchema = UnionSchema::new(&[]);

        assert_eq!(
            DEF.export("test").to_zod_string(),
            "const test = z.lazy(() => z.union([]));"
        );
        assert_eq!(DEF.export("test").to_ts_string(), "type test = never;");
    }
}
