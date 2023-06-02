use std::fmt::Display;

use super::ZodTypeInner;
use crate::formatter::{TsFormatter, ZodFormatter};

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodBool;

impl Display for ZodFormatter<'_, ZodBool> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.bool()")
    }
}

impl Display for TsFormatter<'_, ZodBool> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("bool")
    }
}

impl<Io> From<ZodBool> for ZodTypeInner<Io> {
    fn from(value: ZodBool) -> Self {
        ZodTypeInner::Bool(value)
    }
}

#[cfg(test)]
mod test {

    use crate::formatter::Formatter;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn fmt_ok() {
        assert_eq!(ZodBool.as_zod().to_string(), "z.bool()");
        assert_eq!(ZodBool.as_ts().to_string(), "bool");
    }
}
