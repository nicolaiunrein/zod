use std::fmt::Display;

use super::{Delimited, Formatter, GenericArgument, NamedField, Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Typed {
    Object(&'static [NamedField]),
    Tuple(&'static [InlineSchema]),
    Union(&'static [InlineSchema]),
    DiscriminatedUnion {
        key: &'static str,
        variants: &'static [&'static [NamedField]],
    },
}

impl Typed {
    pub fn is_interface(&self) -> bool {
        match self {
            Typed::Object(_) => true,
            Typed::Tuple(_) => false,
            Typed::Union(_) => false,
            Typed::DiscriminatedUnion { .. } => false,
        }
    }
}

impl Formatter for Typed {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Typed::Object(fields) => {
                f.write_str("z.object({ ")?;
                fields
                    .iter()
                    .comma_separated(f, |f, field| field.fmt_zod(f))?;

                f.write_str(" })")?;
                Ok(())
            }
            Typed::Tuple(fields) => {
                f.write_str("z.tuple([")?;
                fields
                    .iter()
                    .comma_separated(f, |f, field| field.fmt_zod(f))?;

                f.write_str("])")?;
                Ok(())
            }

            Typed::Union(fields) => {
                f.write_str("z.union([")?;
                fields
                    .iter()
                    .comma_separated(f, |f, field| field.fmt_zod(f))?;

                f.write_str("])")?;
                Ok(())
            }

            Typed::DiscriminatedUnion { key, variants } => {
                f.write_fmt(format_args!("z.discriminatedUnion(\"{key}\", ["))?;

                variants
                    .iter()
                    .comma_separated(f, |f, fields| Self::Object(fields).fmt_zod(f))?;

                f.write_str("])")?;
                Ok(())
            }
        }
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Typed::Object(fields) => {
                f.write_str("{ ")?;
                fields
                    .iter()
                    .comma_separated(f, |f, field| field.fmt_ts(f))?;
                f.write_str(" }")?;
                Ok(())
            }
            Typed::Tuple(fields) => {
                f.write_str("[")?;
                fields
                    .iter()
                    .comma_separated(f, |f, field| field.fmt_ts(f))?;
                f.write_str("]")?;
                Ok(())
            }

            Typed::Union(fields) => {
                fields
                    .iter()
                    .fmt_delimited(f, " | ", |f, field| field.fmt_ts(f))?;
                Ok(())
            }
            Typed::DiscriminatedUnion { variants, .. } => {
                variants
                    .iter()
                    .fmt_delimited(f, " | ", |f, fields| Self::Object(fields).fmt_ts(f))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExportSchema {
    Raw {
        args: &'static [GenericArgument],
        ts: &'static str,
        zod: &'static str,
    },
    Typed(Typed),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InlineSchema {
    Ref {
        path: Path,
        args: &'static [InlineSchema],
    },
    Typed(Typed),
}

impl InlineSchema {
    pub const fn path(&self) -> Option<Path> {
        match self {
            InlineSchema::Ref { path, .. } => Some(*path),
            InlineSchema::Typed(_) => None,
        }
    }
}

impl Formatter for InlineSchema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InlineSchema::Ref { path, args } => {
                path.fmt(f)?;
                if !args.is_empty() {
                    f.write_str("(")?;
                    args.iter().comma_separated(f, |f, arg| arg.fmt_zod(f))?;

                    f.write_str(")")?;
                }
            }
            InlineSchema::Typed(typed) => {
                typed.fmt_zod(f)?;
            }
        }
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InlineSchema::Ref { path, args } => {
                path.fmt(f)?;
                if !args.is_empty() {
                    f.write_str("<")?;
                    args.iter().comma_separated(f, |f, arg| arg.fmt_ts(f))?;
                    f.write_str(">")?;
                }
            }
            InlineSchema::Typed(typed) => typed.fmt_ts(f)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Node;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn tuple_ok() {
        const TYPED: Typed = Typed::Tuple(&[
            String::DEFINITION.inline(),
            crate::types::Usize::DEFINITION.inline(),
        ]);
        assert_eq!(TYPED.to_zod_string(), "z.tuple([Rs.String, Rs.Usize])");
        assert_eq!(TYPED.to_ts_string(), "[Rs.String, Rs.Usize]");
    }

    #[test]
    fn object_ok() {
        const TYPED: Typed = Typed::Object(&[
            NamedField::new::<String>("a"),
            NamedField::new::<crate::types::Usize>("b"),
        ]);
        assert_eq!(
            TYPED.to_zod_string(),
            "z.object({ a: Rs.String, b: Rs.Usize })"
        );
        assert_eq!(TYPED.to_ts_string(), "{ a: Rs.String, b: Rs.Usize }");
    }

    #[test]
    fn union_ok() {
        const TYPED: Typed = Typed::Union(&[
            String::DEFINITION.inline(),
            crate::types::Usize::DEFINITION.inline(),
        ]);
        assert_eq!(TYPED.to_zod_string(), "z.union([Rs.String, Rs.Usize])");
        assert_eq!(TYPED.to_ts_string(), "Rs.String | Rs.Usize");
    }

    #[test]
    fn discriminated_union_ok() {
        const TYPED: Typed = Typed::DiscriminatedUnion {
            key: "myKey",
            variants: &[
                &[
                    NamedField::new::<String>("myKey"),
                    NamedField::new::<crate::types::Usize>("b"),
                ],
                &[
                    NamedField::new::<String>("myKey"),
                    NamedField::new::<crate::types::Isize>("c"),
                ],
            ],
        };
        assert_eq!(TYPED.to_zod_string(), "z.discriminatedUnion(\"myKey\", [z.object({ myKey: Rs.String, b: Rs.Usize }), z.object({ myKey: Rs.String, c: Rs.Isize })])");
        assert_eq!(
            TYPED.to_ts_string(),
            "{ myKey: Rs.String, b: Rs.Usize } | { myKey: Rs.String, c: Rs.Isize }"
        );
    }
}
