mod fields;
mod generics;
mod literal;
mod r#struct;
mod r#type;

use std::fmt::Display;

pub use fields::*;
pub use generics::*;
pub use literal::*;
pub use r#struct::*;
pub use r#type::*;

use crate::Namespace;

/// Example:
/// ```
/// # use zod_core::ast::*;
/// inventory::submit!(ZodDefinition::Struct(Struct {
///     ns: "abc",
///     ty: Type {
///         ident: "test",
///         generics: &[Generic::Type { ident: "T1" }, Generic::Type { ident: "T2" }]
///     },
///     fields: StructFields::Named(&[MaybeFlatField::Flat(FlatField {
///         value: FieldValue::Qualified(QualifiedType {
///             ns: "Other",
///             ident: "xx",
///             generics: &[]
///         })
///     })])
/// }));
/// ```
#[derive(Clone, Copy, Debug)]
pub enum ZodDefinition {
    Struct(Struct),
    Literal(Literal),
}

inventory::collect!(ZodDefinition);

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

struct Delimited<I>(pub I, pub &'static str);

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
