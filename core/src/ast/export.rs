use std::fmt::Display;

use super::{Delimited, Docs, ExportSchema, Formatter, GenericArgument, Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Export {
    pub docs: Option<Docs>,
    pub path: Path,
    pub schema: ExportSchema,
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
            ExportSchema::Raw { args, zod, .. } => {
                if !args.is_empty() {
                    f.write_str("(")?;
                    args.iter()
                        .filter(|arg| !matches!(arg, GenericArgument::Assign { .. }))
                        .comma_separated(f, |f, arg| arg.fmt_zod(f))?;
                    f.write_str(") => ")?;
                }
                f.write_str(zod)?;
            }
            ExportSchema::Typed(typed) => typed.fmt_zod(f)?,
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
            ExportSchema::Raw { args, ts, .. } => {
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

            ExportSchema::Typed(typed) => {
                if typed.is_interface() {
                    f.write_str("interface ")?;
                    f.write_str(self.path.name())?;
                    f.write_str(" ")?;
                    typed.fmt_ts(f)?;
                } else {
                    f.write_str("type ")?;
                    f.write_str(self.path.name())?;
                    f.write_str(" = ")?;
                    typed.fmt_ts(f)?;
                    f.write_str(";")?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ast::{NamedField, Node, ObjectSchema, TupleSchema, Typed};
    use crate::Namespace;

    use super::*;
    use pretty_assertions::assert_eq;

    struct Ns;
    impl Namespace for Ns {
        const NAME: &'static str = "Ns";
        const DOCS: Option<&'static str> = None;
        type UniqueMembers = ();
    }

    #[test]
    fn object_ok() {
        const OBJECT: Typed = Typed::Object(ObjectSchema::new(&[
            NamedField::new::<String>("a"),
            NamedField::new::<crate::types::Usize>("b"),
        ]));

        const EXPORT_OBJECT: Export = Export {
            docs: None,
            path: Path::new::<Ns>("test"),
            schema: ExportSchema::Typed(OBJECT),
        };

        assert_eq!(
            EXPORT_OBJECT.to_zod_string(),
            format!(
                "export const test = z.lazy(() => {});",
                OBJECT.to_zod_string()
            )
        );
        assert_eq!(
            EXPORT_OBJECT.to_ts_string(),
            format!("export interface test {}", OBJECT.to_ts_string())
        );
    }

    #[test]
    fn tuple_ok() {
        const TUPLE: Typed = Typed::Tuple(TupleSchema::new(&[
            String::DEFINITION.inline(),
            crate::types::Usize::DEFINITION.inline(),
        ]));

        const EXPORT_TUPLE: Export = Export {
            docs: None,
            path: Path::new::<Ns>("test"),
            schema: ExportSchema::Typed(TUPLE),
        };

        assert_eq!(
            EXPORT_TUPLE.to_zod_string(),
            format!(
                "export const test = z.lazy(() => {});",
                TUPLE.to_zod_string()
            )
        );
        assert_eq!(
            EXPORT_TUPLE.to_ts_string(),
            format!("export type test = {};", TUPLE.to_ts_string())
        );
    }
}
