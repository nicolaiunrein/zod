use std::fmt::Display;

use super::{Delimited, Formatter, GenericArgument, NamedField, Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Typed {
    Object(&'static [NamedField]),
    Tuple(&'static [InlineSchema]),
}

impl Typed {
    pub fn is_interface(&self) -> bool {
        match self {
            Typed::Object(_) => true,
            Typed::Tuple(_) => false,
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
}
