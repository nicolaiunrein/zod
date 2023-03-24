use std::fmt::Display;

use super::{Delimited, Docs, ExportSchema, Formatter, GenericArgument, Path};

/// The struct containing all the info about a [Node](crate::Node) to be exported
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
            ExportSchema::Object(inner) => inner.fmt_zod(f)?,
            ExportSchema::Newtype(inner) => inner.fmt_zod(f)?,
            ExportSchema::Tuple(inner) => inner.fmt_zod(f)?,
            ExportSchema::Union(inner) => inner.fmt_zod(f)?,
            ExportSchema::DiscriminatedUnion(inner) => inner.fmt_zod(f)?,
        }

        f.write_str(");")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(docs) = self.docs {
            docs.fmt_ts(f)?;
        }

        f.write_str("export ")?;

        let mut fmt_type = |inner: &dyn Formatter| {
            f.write_str("type ")?;
            f.write_str(self.path.name())?;
            f.write_str(" = ")?;
            inner.fmt_ts(f)?;
            f.write_str(";")?;
            std::fmt::Result::Ok(())
        };

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
            ExportSchema::Object(inner) => {
                f.write_str("interface ")?;
                f.write_str(self.path.name())?;
                f.write_str(" ")?;
                inner.fmt_ts(f)?;
            }
            ExportSchema::Tuple(inner) => {
                fmt_type(&inner)?;
            }

            ExportSchema::Newtype(inner) => {
                fmt_type(&inner)?;
            }

            ExportSchema::Union(inner) => {
                fmt_type(&inner)?;
            }
            ExportSchema::DiscriminatedUnion(inner) => {
                fmt_type(&inner)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ast::{NamedField, ObjectSchema, TupleField, TupleSchema};
    use crate::Namespace;

    use super::*;
    use pretty_assertions::assert_eq;

    struct Ns;
    impl Namespace for Ns {
        const NAME: &'static str = "Ns";
        const DOCS: Option<Docs> = None;
    }

    #[test]
    fn object_ok() {
        const OBJECT: ObjectSchema = ObjectSchema::new(&[
            NamedField::new::<String>("a"),
            NamedField::new::<crate::types::Usize>("b"),
        ]);

        const EXPORT_OBJECT: Export = Export {
            docs: None,
            path: Path::new::<Ns>("test"),
            schema: ExportSchema::Object(OBJECT),
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
        const TUPLE: TupleSchema = TupleSchema::new(&[
            TupleField::new::<String>(),
            TupleField::new::<crate::types::Usize>(),
        ]);

        const EXPORT_TUPLE: Export = Export {
            docs: None,
            path: Path::new::<Ns>("test"),
            schema: ExportSchema::Tuple(TUPLE),
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
