use std::fmt::Display;

use super::{Docs, ExportSchema, Formatter, Path};

/// The struct containing all the info about a [RequestType](crate::RequestType)/[ResponseType](crate::ResponseType) to be exported
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

        f.write_str("export ")?;

        // todo remove
        match self.schema {
            ExportSchema::Raw(_) => {}
            ExportSchema::Object(_) => {}
            _ => {
                f.write_str("const ")?;
                f.write_str(self.path.name())?;
                f.write_str(" = ")?;
            }
        }

        match self.schema {
            ExportSchema::Raw(schema) => (self.path.name(), schema).fmt_zod(f)?,
            ExportSchema::Object(schema) => (self.path.name(), schema).fmt_zod(f)?,

            //todo generics
            ExportSchema::Newtype(inner) => {
                f.write_str("z.lazy(() => ")?;
                inner.fmt_zod(f)?;
                f.write_str(");")?;
            }

            //todo generics
            ExportSchema::Tuple(inner) => {
                f.write_str("z.lazy(() => ")?;
                inner.fmt_zod(f)?;
                f.write_str(");")?;
            }

            //todo generics
            ExportSchema::Union(inner) => {
                f.write_str("z.lazy(() => ")?;
                inner.fmt_zod(f)?;
                f.write_str(");")?;
            }
            //todo generics
            ExportSchema::DiscriminatedUnion(inner) => {
                f.write_str("z.lazy(() => ")?;
                inner.fmt_zod(f)?;
                f.write_str(");")?;
            }
        }

        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(docs) = self.docs {
            docs.fmt_ts(f)?;
        }

        f.write_str("export ")?;

        // todo remove
        let mut fmt_type = |inner: &dyn Formatter| {
            f.write_str("type ")?;
            f.write_str(self.path.name())?;
            f.write_str(" = ")?;
            inner.fmt_ts(f)?;
            f.write_str(";")?;
            std::fmt::Result::Ok(())
        };

        match self.schema {
            ExportSchema::Raw(schema) => (self.path.name(), schema).fmt_ts(f)?,
            ExportSchema::Object(inner) => (self.path.name(), inner).fmt_ts(f)?,

            //todo generics
            ExportSchema::Tuple(inner) => fmt_type(&inner)?,

            //todo generics
            ExportSchema::Newtype(inner) => fmt_type(&inner)?,

            //todo generics
            ExportSchema::Union(inner) => fmt_type(&inner)?,

            //todo generics
            ExportSchema::DiscriminatedUnion(inner) => fmt_type(&inner)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ast::{NewtypeSchema, TupleField, TupleSchema};
    use crate::Namespace;

    use super::*;
    use pretty_assertions::assert_eq;

    struct Ns;
    impl Namespace for Ns {
        const NAME: &'static str = "Ns";
        const DOCS: Option<Docs> = None;
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

    #[test]
    fn newtype_ok() {
        const NEWTYPE: NewtypeSchema =
            NewtypeSchema::new(&crate::ast::Ref::new_req::<String>(), false);

        const EXPORT_TUPLE: Export = Export {
            docs: None,
            path: Path::new::<Ns>("test"),
            schema: ExportSchema::Newtype(NEWTYPE),
        };

        assert_eq!(
            EXPORT_TUPLE.to_zod_string(),
            format!(
                "export const test = z.lazy(() => {});",
                NEWTYPE.to_zod_string()
            )
        );
        assert_eq!(
            EXPORT_TUPLE.to_ts_string(),
            format!("export type test = {};", NEWTYPE.to_ts_string())
        );
    }
}
