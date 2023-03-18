use super::{Delimited, FormatTypescript, FormatZod, Generic};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Type {
    pub ident: &'static str,
    pub generics: &'static [Generic],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct QualifiedType {
    pub ns: &'static str,
    pub ident: &'static str,
    pub generics: &'static [Generic],
}

impl Type {
    pub const fn qualify(self, ns: &'static str) -> QualifiedType {
        QualifiedType {
            ns,
            ident: self.ident,
            generics: self.generics,
        }
    }
}

impl QualifiedType {
    pub const fn into_type(self) -> Type {
        Type {
            ident: self.ident,
            generics: self.generics,
        }
    }
}

impl FormatZod for QualifiedType {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ns)?;
        f.write_str(".")?;
        f.write_str(self.ident)?;
        if !self.generics.is_empty() {
            f.write_str("(")?;
            Delimited(self.generics, ", ").fmt_zod(f)?;
            f.write_str(")")?;
        }

        Ok(())
    }
}

impl FormatTypescript for QualifiedType {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ns)?;
        f.write_str(".")?;
        f.write_str(self.ident)?;
        if !self.generics.is_empty() {
            f.write_str("<")?;
            Delimited(self.generics, ", ").fmt_zod(f)?;
            f.write_str(">")?;
        }

        Ok(())
    }
}

impl FormatZod for Type {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ident)?;
        if !self.generics.is_empty() {
            f.write_str("(")?;
            Delimited(self.generics, ", ").fmt_zod(f)?;
            f.write_str(")")?;
        }

        Ok(())
    }
}

impl FormatTypescript for Type {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ident)?;
        if !self.generics.is_empty() {
            f.write_str("<")?;
            Delimited(self.generics, ", ").fmt_zod(f)?;
            f.write_str(">")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Generic;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn zod_type() {
        let ty = Type {
            ident: "abc",
            generics: Default::default(),
        };

        assert_eq!(ty.to_zod_string(), "abc");
    }

    #[test]
    fn zod_qualified_type() {
        let ty = QualifiedType {
            ns: "Ns",
            ident: "abc",
            generics: Default::default(),
        };

        assert_eq!(ty.to_zod_string(), "Ns.abc");
    }

    #[test]
    fn type_with_generics() {
        let ty = Type {
            ident: "abc",
            generics: &[Generic::Type { ident: "A" }, Generic::Type { ident: "B" }],
        };

        assert_eq!(ty.to_zod_string(), "abc(A, B)");
    }

    #[test]
    fn qualified_type_with_generics() {
        let ty = QualifiedType {
            ns: "Ns",
            ident: "abc",
            generics: &[Generic::Type { ident: "A" }, Generic::Type { ident: "B" }],
        };

        assert_eq!(ty.to_zod_string(), "Ns.abc(A, B)");
    }
}
