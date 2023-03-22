use std::fmt::Display;

use super::{Formatter, Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GenericArgument {
    Type(&'static str),
    Const {
        name: &'static str,
        path: Path,
    },
    Assign {
        name: &'static str,
        value: &'static str,
    },
}

impl Formatter for GenericArgument {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericArgument::Type(name) => {
                f.write_str(name)?;
                f.write_str(": ")?;
                f.write_str("z.ZodTypeAny")?;
            }

            GenericArgument::Const { name, path } => {
                f.write_str(name)?;
                f.write_str(": ")?;
                path.fmt(f)?;
            }
            GenericArgument::Assign { .. } => {}
        }
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericArgument::Type(name) => f.write_str(name),
            GenericArgument::Assign { name, value } => {
                f.write_str(name)?;
                f.write_str(" = ")?;
                f.write_str(value)?;
                Ok(())
            }
            GenericArgument::Const { name, path } => {
                f.write_str(name)?;
                f.write_str(" extends ")?;
                path.fmt(f)?;
                Ok(())
            }
        }
    }
}
