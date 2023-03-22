use super::{Formatter, InlineSchema};

/// Representation of a generic argument in typescript/zod
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GenericArgument {
    /// Example: the `T` in `Vec<T>`
    Type(&'static str),

    /// Example: the `N: Rs.Usize` in `Array<N: Rs.Usize>`
    Const {
        name: &'static str,
        schema: InlineSchema,
    },

    /// Example: the `Def = [T, ...T[]]` in `Array<T, N: Rs.Usize, Def = [T, ..T[]]>`
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

            GenericArgument::Const { name, schema } => {
                f.write_str(name)?;
                f.write_str(": ")?;
                schema.fmt_zod(f)?;
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
            GenericArgument::Const { name, schema } => {
                f.write_str(name)?;
                f.write_str(" extends ")?;
                schema.fmt_ts(f)?;
                Ok(())
            }
        }
    }
}
