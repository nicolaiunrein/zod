use std::fmt::Display;

use super::{Delimited, Formatter, GenericArgument, NamedField, Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Schema {
    Raw {
        args: &'static [GenericArgument],
        ts: &'static str,
        zod: &'static str,
    },
    Object(&'static [NamedField]),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InlineSchema {
    Ref(Path),
    Generic {
        path: Path,
        args: &'static [InlineSchema],
    },
    Object(&'static [NamedField]),
}

impl InlineSchema {
    pub const fn path(&self) -> Option<Path> {
        match self {
            InlineSchema::Ref(path) => Some(*path),
            InlineSchema::Generic { path, .. } => Some(*path),
            InlineSchema::Object(_) => None,
        }
    }
}

impl Formatter for InlineSchema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InlineSchema::Ref(path) => {
                path.fmt(f)?;
            }
            InlineSchema::Generic { path, args } => {
                path.fmt(f)?;
                f.write_str("(")?;
                args.iter().comma_separated(f, |f, arg| arg.fmt_zod(f))?;

                f.write_str(")")?;
            }
            InlineSchema::Object(fields) => {
                f.write_str("z.object({ ")?;
                fields
                    .iter()
                    .comma_separated(f, |f, field| field.fmt_zod(f))?;

                f.write_str(" })")?;
            }
        }
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InlineSchema::Ref(path) => path.fmt(f)?,
            InlineSchema::Generic { path, args } => {
                path.fmt(f)?;
                if !args.is_empty() {
                    f.write_str("<")?;
                    args.iter().comma_separated(f, |f, arg| arg.fmt_ts(f))?;
                    f.write_str(">")?;
                }
            }
            InlineSchema::Object(fields) => {
                f.write_str("{ ")?;
                fields
                    .iter()
                    .comma_separated(f, |f, field| field.fmt_ts(f))?;
                f.write_str(" }")?;
            }
        }
        Ok(())
    }
}

impl Formatter for Schema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Schema::Raw { args, zod, .. } => {
                if !args.is_empty() {
                    f.write_str("(")?;
                    args.iter()
                        .filter(|arg| !matches!(arg, GenericArgument::Assign { .. }))
                        .comma_separated(f, |f, arg| arg.fmt_zod(f))?;
                    f.write_str(") => ")?;
                }
                f.write_str(zod)?;
            }
            Schema::Object(fields) => {
                f.write_str("z.object({ ")?;
                fields
                    .iter()
                    .comma_separated(f, |f, field| field.fmt_zod(f))?;

                f.write_str(" })")?;
            }
        }
        Ok(())
    }
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Schema::Raw { args, ts, .. } => {
                if !args.is_empty() {
                    f.write_str("<")?;
                    args.iter().comma_separated(f, |f, arg| arg.fmt_zod(f))?;
                    f.write_str("> => ")?;
                }
                f.write_str(ts)?;
            }
            Schema::Object(fields) => {
                f.write_str(" { ")?;
                fields
                    .iter()
                    .comma_separated(f, |f, field| field.fmt_ts(f))?;
                f.write_str(" }")?;
            }
        }
        Ok(())
    }
}
