use std::fmt::Display;

use super::{Compiler, Docs, ExportSchema, Path};

/// The struct containing all the info about a [RequestType](crate::RequestType)/[ResponseType](crate::ResponseType) to be exported
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Export {
    pub docs: Option<Docs>,
    pub path: Path,
    pub schema: ExportSchema,
}

impl Display for Export {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("    ")?;
        self.fmt_ts(f)?;
        f.write_str("\n")?;
        f.write_str("    ")?;
        self.fmt_zod(f)?;
        Ok(())
    }
}

impl Compiler for Export {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(docs) = self.docs {
            docs.fmt_zod(f)?;
        }

        f.write_str("export ")?;

        let name = self.path.name();
        match self.schema {
            ExportSchema::Raw(schema) => schema.export(name).fmt_zod(f)?,
            ExportSchema::Object(schema) => schema.export(name).fmt_zod(f)?,

            //todo generics
            ExportSchema::Newtype(schema) => schema.export(name).fmt_zod(f)?,
            ExportSchema::Tuple(schema) => schema.export(name).fmt_zod(f)?,
            ExportSchema::Union(schema) => schema.export(name).fmt_zod(f)?,
            ExportSchema::DiscriminatedUnion(schema) => schema.export(name).fmt_zod(f)?,
        }

        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(docs) = self.docs {
            docs.fmt_ts(f)?;
        }

        f.write_str("export ")?;

        let name = self.path.name();
        match self.schema {
            ExportSchema::Raw(schema) => schema.export(name).fmt_ts(f)?,
            ExportSchema::Object(schema) => schema.export(name).fmt_ts(f)?,

            //todo generics
            ExportSchema::Newtype(schema) => schema.export(name).fmt_ts(f)?,
            ExportSchema::Tuple(schema) => schema.export(name).fmt_ts(f)?,
            ExportSchema::Union(schema) => schema.export(name).fmt_ts(f)?,
            ExportSchema::DiscriminatedUnion(schema) => schema.export(name).fmt_ts(f)?,
        }
        Ok(())
    }
}
