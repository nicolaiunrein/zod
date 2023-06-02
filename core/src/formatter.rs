use std::ops::Deref;
pub struct ZodFormatter<'a, T>(pub &'a T);

pub struct TsFormatter<'a, T>(pub &'a T);

impl<T> Deref for ZodFormatter<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T> Deref for TsFormatter<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

pub trait Formatter
where
    Self: Sized,
{
    fn as_zod(&self) -> ZodFormatter<Self> {
        ZodFormatter(self)
    }
    fn as_ts(&self) -> TsFormatter<Self> {
        TsFormatter(self)
    }
}

impl<T> Formatter for T
where
    T: Sized,
    for<'a> ZodFormatter<'a, Self>: std::fmt::Display,
    for<'a> TsFormatter<'a, Self>: std::fmt::Display,
{
}
