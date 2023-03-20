use super::{Delimited, FormatInlined, FormatTypescript, FormatZod, Generic};

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

pub struct TypeName(QualifiedType);
pub struct TypeArg(QualifiedType);

impl QualifiedType {
    pub const fn as_arg(self) -> TypeArg {
        TypeArg(self)
    }

    pub const fn as_name(self) -> TypeName {
        TypeName(self)
    }
}

impl FormatInlined for QualifiedType {
    fn fmt_inlined(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ns)?;
        f.write_str(".")?;
        f.write_str(self.ident)?;

        if !self.generics.is_empty() {
            f.write_str("(")?;
            let tys = self
                .generics
                .into_iter()
                .map(|gen| gen.resolved.ty())
                .collect::<Vec<_>>();

            Delimited(tys.as_slice(), ", ").fmt_inlined(f)?;
            f.write_str(")")?;
        }

        Ok(())
    }
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

impl FormatZod for TypeArg {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.ns)?;
        f.write_str(".")?;
        f.write_str(self.0.ident)?;
        if !self.0.generics.is_empty() {
            f.write_str("(")?;
            Delimited(self.0.generics, ", ").fmt_zod(f)?;
            f.write_str(")")?;
        }

        Ok(())
    }
}

impl FormatTypescript for TypeArg {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.ns)?;
        f.write_str(".")?;
        f.write_str(self.0.ident)?;
        if !self.0.generics.is_empty() {
            f.write_str("<")?;
            Delimited(self.0.generics, ", ").fmt_zod(f)?;
            f.write_str(">")?;
        }

        Ok(())
    }
}

impl FormatZod for TypeName {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.ident)?;
        if !self.0.generics.is_empty() {
            f.write_str("(")?;
            Delimited(self.0.generics, ", ").fmt_zod(f)?;
            f.write_str(")")?;
        }

        Ok(())
    }
}

impl FormatTypescript for TypeName {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.ident)?;
        if !self.0.generics.is_empty() {
            f.write_str("<")?;
            Delimited(self.0.generics, ", ").fmt_zod(f)?;
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

        assert_eq!(ty.as_arg().to_zod_string(), "Ns.abc");
    }

    #[test]
    fn qualified_type_with_generics() {
        const TY: QualifiedType = QualifiedType {
            ns: "Ns",
            ident: "abc",
            generics: &[Generic::new_for::<()>("A"), Generic::new_for::<()>("B")],
        };

        assert_eq!(TY.as_arg().to_zod_string(), "Ns.abc(A, B)");
    }
}
