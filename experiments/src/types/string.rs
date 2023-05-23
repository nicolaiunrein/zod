use std::fmt::Display;

use super::{Ts, Zod};

pub struct ZodString;

impl Display for Zod<'_, ZodString> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.string()")
    }
}

impl Display for Ts<'_, ZodString> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("string")
    }
}
