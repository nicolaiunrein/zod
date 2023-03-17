mod fields;
mod generics;
mod literal;
mod r#struct;
mod r#type;

use std::fmt::Display;

use const_format::concatcp;
pub use fields::*;
pub use generics::*;
pub use literal::*;
pub use r#struct::*;
pub use r#type::*;

use crate::Namespace;

/// Example:
/// ```
/// # use zod_core::ast::*;
/// inventory::submit!(Item::Struct(Struct {
///     ns: "abc",
///     ty: Type {
///         ident: "test",
///         generics: &[Generic::Type { ident: "T1" }, Generic::Type { ident: "T2" }]
///     },
///     fields: StructFields::Named(&[AnyNamedField::Flat(FlatField {
///         value: FieldValue::Qualified(QualifiedType {
///             ns: "Other",
///             ident: "xx",
///             generics: &[]
///         })
///     })])
/// }));
/// ```
#[derive(Clone, Copy, Debug)]
pub enum Item {
    Struct(Struct),
    Literal(Literal),
}

inventory::collect!(Item);

impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_zod(f)?;
        f.write_str("\n")?;
        self.fmt_ts(f)?;
        Ok(())
    }
}

impl Item {
    pub fn is_member_of<T: Namespace + ?Sized + 'static>(&self) -> bool {
        match self {
            Item::Struct(inner) => T::NAME == inner.ns,
            Item::Literal(inner) => T::NAME == inner.ns,
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Item::Struct(inner) => inner.ty.ident,
            Item::Literal(inner) => inner.ty.ident,
        }
    }

    pub const fn ty(&self) -> Type {
        match self {
            Item::Struct(inner) => inner.ty,
            Item::Literal(inner) => inner.ty,
        }
    }

    pub const fn ns(&self) -> &'static str {
        match self {
            Item::Struct(inner) => inner.ns,
            Item::Literal(inner) => inner.ns,
        }
    }
}

impl FormatZod for Item {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Struct(inner) => {
                f.write_str("export ")?;
                inner.fmt_zod(f)?;
            }
            Item::Literal(inner) => {
                f.write_str("export ")?;
                inner.fmt_zod(f)?;
            }
        }
        Ok(())
    }
}

impl FormatTypescript for Item {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Struct(inner) => {
                f.write_str("export ")?;
                inner.fmt_ts(f)?;
            }
            Item::Literal(inner) => {
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
