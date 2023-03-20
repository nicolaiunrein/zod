use super::{FormatTypescript, FormatZod};

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

#[cfg(test)]
mod test {
    use crate::ZodType;
    use pretty_assertions::assert_eq;

    #[test]
    fn inline() {
        type T = Vec<String>;

        assert_eq!(T::INLINED.to_string(), "Rs.Vec(Rs.String)")
    }
}
