use std::fmt::Display;

pub trait Formatter {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

    fn to_zod_string(&self) -> String
    where
        Self: Sized,
    {
        struct Helper<'a>(&'a dyn Formatter);
        impl Display for Helper<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt_zod(f)
            }
        }

        Helper(self).to_string()
    }

    fn to_ts_string(&self) -> String
    where
        Self: Sized,
    {
        struct Helper<'a>(&'a dyn Formatter);
        impl Display for Helper<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt_ts(f)
            }
        }

        Helper(self).to_string()
    }
}
