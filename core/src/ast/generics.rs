use crate::ZodType;

use super::{FormatTypescript, FormatZod, ZodDefinition};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Generic {
    pub name: &'static str,
    pub resolved: &'static ZodDefinition,
}

impl Generic {
    pub const fn new_for<T: ZodType>(name: &'static str) -> Self {
        Self {
            name,
            resolved: &T::AST.def,
        }
    }
}

impl FormatZod for Generic {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

impl FormatTypescript for Generic {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

#[cfg(test)]
mod test {
    use crate::ast::FormatResolvedZod;
    use crate::ZodType;
    use pretty_assertions::assert_eq;

    #[test]
    fn inline() {
        type T = Vec<String>;

        assert_eq!(
            T::AST.def.ty().to_resolved_zod_string(),
            "Rs.Vec(Rs.String)"
        )
    }
}
