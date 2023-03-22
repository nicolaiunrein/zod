use std::fmt::Display;

use super::{Delimited, Docs, Formatter, GenericArgument, Path, Schema};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Export {
    pub docs: Option<Docs>,
    pub path: Path,
    pub schema: Schema,
}

impl Display for Export {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ts(f)?;
        f.write_str("\n")?;
        self.fmt_zod(f)?;
        Ok(())
    }
}

impl Formatter for Export {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(docs) = self.docs {
            docs.fmt_zod(f)?;
        }
        f.write_str("export const ")?;
        f.write_str(self.path.name())?;
        f.write_str(" = z.lazy(() => ")?;

        match self.schema {
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
            Schema::Typed(typed) => typed.fmt_zod(f)?,
        }

        f.write_str(");")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(docs) = self.docs {
            docs.fmt_ts(f)?;
        }

        f.write_str("export ")?;
        match self.schema {
            Schema::Raw { args, ts, .. } => {
                f.write_str("type ")?;
                f.write_str(self.path.name())?;
                if !args.is_empty() {
                    f.write_str("<")?;
                    args.iter().comma_separated(f, |f, arg| arg.fmt_ts(f))?;
                    f.write_str(">")?;
                }
                f.write_str(" = ")?;
                f.write_str(ts)?;
                f.write_str(";")?;
            }

            Schema::Typed(typed) => {
                if typed.is_interface() {
                    f.write_str("interface ")?;
                    f.write_str(self.path.name())?;
                    f.write_str(" ")?;
                    typed.fmt_ts(f)?;
                } else {
                    f.write_str("const ")?;
                    f.write_str(self.path.name())?;
                    f.write_str(" ")?;
                    typed.fmt_ts(f)?;
                }
            }
        }
        Ok(())
    }
}
