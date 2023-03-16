mod fields;
mod generics;
mod r#struct;
mod r#type;

use std::fmt::Display;

pub use fields::*;
pub use generics::*;
pub use r#struct::*;
pub use r#type::*;

struct ZodTypeAny;

impl Display for ZodTypeAny {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.ZodTypeAny")
    }
}

pub enum Item {
    Struct(Struct),
}

struct ZodFormatter<'a, T: FormatZod>(&'a T);
struct TsFormatter<'a, T: FormatTypescript>(&'a T);

impl FormatZod for Item {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Struct(inner) => {
                f.write_str("export ")?;
                inner.fmt_zod(f)?;
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

trait FormatZod {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn to_zod_string(&self) -> String
    where
        Self: Sized,
    {
        ZodFormatter(self).to_string()
    }
}

trait FormatTypescript {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn to_ts_string(&self) -> String
    where
        Self: Sized,
    {
        TsFormatter(self).to_string()
    }
}
