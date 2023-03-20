use crate::ZodType;

use super::{FormatTypescript, FormatZod, ZodDefinition};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GenericName {
    Type {
        ident: &'static str,
    },
    QualifiedType {
        ns: &'static str,
        ident: &'static str,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Generic {
    pub name: GenericName,
    pub resolved: &'static ZodDefinition,
}

impl Generic {
    pub const fn new_for<T: ZodType>(name: &'static str) -> Self {
        Self {
            name: GenericName::Type { ident: name },
            resolved: &T::AST.def,
        }
    }
}

impl FormatZod for GenericName {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type { ident } => f.write_str(ident),
            Self::QualifiedType { ns, ident } => {
                f.write_str(ns)?;
                f.write_str(".")?;
                f.write_str(ident)
            }
        }
    }
}

impl FormatZod for Generic {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt_zod(f)
    }
}

impl FormatTypescript for GenericName {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type { ident } => f.write_str(ident),
            Self::QualifiedType { ns, ident } => {
                f.write_str(ns)?;
                f.write_str(".")?;
                f.write_str(ident)
            }
        }
    }
}

impl FormatTypescript for Generic {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt_ts(f)
    }
}

#[cfg(test)]
mod test {
    use crate::ast::FormatInlined;
    use crate::ZodType;
    use pretty_assertions::assert_eq;

    #[test]
    fn inline() {
        type T = Vec<String>;

        assert_eq!(T::AST.def.ty().to_inlined_string(), "Rs.Vec(Rs.String)")
    }
}
