use std::fmt::Display;

use super::{Ts, Zod};

pub struct ZodNumber;

impl Display for Zod<'_, ZodNumber> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.number()")
    }
}

impl Display for Ts<'_, ZodNumber> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("number")
    }
}
