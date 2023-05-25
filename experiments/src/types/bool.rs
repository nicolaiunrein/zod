use std::fmt::Display;

use super::{Ts, Zod, ZodTypeInner};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodBool;

impl Display for Zod<'_, ZodBool> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.bool()")
    }
}

impl Display for Ts<'_, ZodBool> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("bool")
    }
}

impl From<ZodBool> for ZodTypeInner {
    fn from(value: ZodBool) -> Self {
        ZodTypeInner::Bool(value)
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn fmt_ok() {
        assert_eq!(Zod(&ZodBool).to_string(), "z.bool()");
        assert_eq!(Ts(&ZodBool).to_string(), "bool");
    }
}
