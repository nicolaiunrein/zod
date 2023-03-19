mod fields;
mod generics;
mod literal;
mod r#struct;
mod r#type;

use std::fmt::Display;

pub(crate) use crate::Delimited;
pub use fields::*;
pub use generics::*;
pub use literal::*;
pub use r#struct::*;
pub use r#type::*;

use crate::Namespace;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ZodExport {
    pub docs: Option<&'static str>,
    pub def: ZodDefinition,
}

impl Display for ZodExport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_zod(f)?;
        f.write_str("\n")?;
        self.fmt_ts(f)?;
        Ok(())
    }
}

impl FormatZod for ZodExport {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(docs) = self.docs {
            f.write_str(&format_docs(docs))?;
        }
        self.def.fmt_zod(f)
    }
}

impl FormatTypescript for ZodExport {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(docs) = self.docs {
            f.write_str(&format_docs(docs))?;
        }
        self.def.fmt_ts(f)
    }
}

impl ZodExport {
    pub fn docs(&self) -> Option<&'static str> {
        self.docs
    }

    pub fn is_member_of<T: Namespace + ?Sized + 'static>(&self) -> bool {
        self.def.is_member_of::<T>()
    }

    pub const fn name(&self) -> &'static str {
        self.def.name()
    }

    pub const fn ty(&self) -> Type {
        self.def.ty()
    }

    pub const fn ns(&self) -> &'static str {
        self.def.ns()
    }

    pub const fn generics(&self) -> &'static [Generic] {
        self.def.generics()
    }

    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.ns(), self.name())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ZodDefinition {
    Struct(Struct),
    Literal(Literal),
}

impl Display for ZodDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_zod(f)?;
        f.write_str("\n")?;
        self.fmt_ts(f)?;
        Ok(())
    }
}

impl ZodDefinition {
    pub fn is_member_of<T: Namespace + ?Sized + 'static>(&self) -> bool {
        match self {
            ZodDefinition::Struct(inner) => T::NAME == inner.ns,
            ZodDefinition::Literal(inner) => T::NAME == inner.ns,
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            ZodDefinition::Struct(inner) => inner.ty.ident,
            ZodDefinition::Literal(inner) => inner.ty.ident,
        }
    }

    pub const fn ty(&self) -> Type {
        match self {
            ZodDefinition::Struct(inner) => inner.ty,
            ZodDefinition::Literal(inner) => inner.ty,
        }
    }

    pub const fn ns(&self) -> &'static str {
        match self {
            ZodDefinition::Struct(inner) => inner.ns,
            ZodDefinition::Literal(inner) => inner.ns,
        }
    }

    pub const fn generics(&self) -> &'static [Generic] {
        match self {
            ZodDefinition::Struct(inner) => inner.ty.generics,
            ZodDefinition::Literal(inner) => inner.ty.generics,
        }
    }
}

impl FormatZod for ZodDefinition {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZodDefinition::Struct(inner) => {
                f.write_str("export ")?;
                inner.fmt_zod(f)?;
            }
            ZodDefinition::Literal(inner) => {
                f.write_str("export ")?;
                inner.fmt_zod(f)?;
            }
        }
        Ok(())
    }
}

impl FormatTypescript for ZodDefinition {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZodDefinition::Struct(inner) => {
                f.write_str("export ")?;
                inner.fmt_ts(f)?;
            }
            ZodDefinition::Literal(inner) => {
                f.write_str("export ")?;
                inner.fmt_ts(f)?;
            }
        }
        Ok(())
    }
}

impl<'a, T> Display for ZodFormatter<'a, T>
where
    T: FormatZod,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt_zod(f)
    }
}

impl<'a, T> Display for TsFormatter<'a, T>
where
    T: FormatTypescript,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt_ts(f)
    }
}

pub trait FormatZod {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn to_zod_string(&self) -> String
    where
        Self: Sized,
    {
        ZodFormatter(self).to_string()
    }
}

pub trait FormatTypescript {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn to_ts_string(&self) -> String
    where
        Self: Sized,
    {
        TsFormatter(self).to_string()
    }
}

struct ZodFormatter<'a, T: FormatZod>(&'a T);
struct TsFormatter<'a, T: FormatTypescript>(&'a T);

impl<T> Display for Delimited<&[T]>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.clone().into_iter().peekable();

        while let Some(item) = iter.next() {
            item.fmt(f)?;
            if iter.peek().is_some() {
                f.write_str(self.1)?;
            }
        }
        Ok(())
    }
}

impl<T> FormatZod for Delimited<&[T]>
where
    T: FormatZod,
{
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.clone().into_iter().peekable();

        while let Some(item) = iter.next() {
            item.fmt_zod(f)?;
            if iter.peek().is_some() {
                f.write_str(self.1)?;
            }
        }
        Ok(())
    }
}

impl<T> FormatTypescript for Delimited<&[T]>
where
    T: FormatTypescript,
{
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.clone().into_iter().peekable();

        while let Some(item) = iter.next() {
            item.fmt_ts(f)?;
            if iter.peek().is_some() {
                f.write_str(self.1)?;
            }
        }
        Ok(())
    }
}

fn format_docs(input: &str) -> String {
    format!(
        "/**\n{}*/\n",
        input
            .lines()
            .map(|line| format!("* {}\n", line))
            .collect::<String>()
    )
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn format_docs_ok() {
        let docs = "First Line\nSecond Line";
        let expected = "/**
* First Line
* Second Line
*/
";
        assert_eq!(format_docs(docs), expected);
    }

    #[test]
    fn docs_export_ok() {
        let export = ZodExport {
            docs: Some("Hallo Welt\nSecond Line"),
            def: ZodDefinition::Struct(Struct {
                ns: "Ns",
                ty: Type {
                    ident: "test",
                    generics: &[],
                },
                fields: StructFields::Named(&[]),
            }),
        };

        assert_eq!(
            export.to_ts_string(),
            format!(
                "{}{}",
                format_docs(export.docs.unwrap()),
                export.def.to_ts_string()
            )
        );

        assert_eq!(
            export.to_zod_string(),
            format!(
                "{}{}",
                format_docs(export.docs.unwrap()),
                export.def.to_zod_string()
            )
        );
    }
}
