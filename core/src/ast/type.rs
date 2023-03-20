use super::{Delimited, FormatInlinedTs, FormatInlinedZod, FormatTypescript, FormatZod, Generic};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeDef {
    pub ns: &'static str,
    pub ident: &'static str,
    pub generics: &'static [Generic],
}

pub struct TypeName(TypeDef);
pub struct TypeArg(TypeDef);

impl TypeDef {
    pub const fn as_arg(self) -> TypeArg {
        TypeArg(self)
    }

    pub const fn as_name(self) -> TypeName {
        TypeName(self)
    }
}

impl FormatInlinedZod for TypeDef {
    fn fmt_inlined_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

            Delimited(tys.as_slice(), ", ").fmt_inlined_zod(f)?;
            f.write_str(")")?;
        }

        Ok(())
    }
}

impl FormatInlinedTs for TypeDef {
    fn fmt_inlined_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ns)?;
        f.write_str(".")?;
        f.write_str(self.ident)?;

        if !self.generics.is_empty() {
            f.write_str("<")?;
            let tys = self
                .generics
                .into_iter()
                .map(|gen| gen.resolved.ty())
                .collect::<Vec<_>>();

            Delimited(tys.as_slice(), ", ").fmt_inlined_ts(f)?;
            f.write_str(">")?;
        }

        Ok(())
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

#[cfg(test)]
mod test {

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn zod_qualified_type() {
        let ty = TypeDef {
            ns: "Ns",
            ident: "abc",
            generics: Default::default(),
        };

        assert_eq!(ty.as_arg().to_zod_string(), "Ns.abc");
    }

    #[test]
    fn qualified_type_with_generics() {
        const TY: TypeDef = TypeDef {
            ns: "Ns",
            ident: "abc",
            generics: &[Generic::new_for::<()>("A"), Generic::new_for::<()>("B")],
        };

        assert_eq!(TY.as_arg().to_zod_string(), "Ns.abc(A, B)");
    }
}
