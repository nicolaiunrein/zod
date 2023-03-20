use std::fmt::Display;

use super::{Delimited, FormatTypescript, FormatZod, Generic, GenericMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeDef {
    pub ns: &'static str,
    pub ident: &'static str,
    pub generics: &'static [Generic],
}

pub struct TypeName(TypeDef);
pub struct TypeArg(TypeDef, GenericMap);

impl TypeDef {
    pub const fn as_arg(self, map: GenericMap) -> TypeArg {
        TypeArg(self, map)
    }

    pub const fn as_name(self) -> TypeName {
        TypeName(self)
    }
}

impl FormatZod for TypeArg {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.ns)?;
        f.write_str(".")?;
        f.write_str(self.0.ident)?;
        if !self.0.generics.is_empty() {
            f.write_str("(")?;
            let gen = self
                .0
                .generics
                .into_iter()
                .enumerate()
                .map(|(index, g)| {
                    self.1
                        .get(&(index as u64))
                        .map(|x| (*x).to_owned())
                        .unwrap_or_else(|| g.resolved.qualified_name())
                })
                .collect::<Vec<_>>();

            Delimited(gen.as_slice(), ", ").fmt(f)?;
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
            let gen = self
                .0
                .generics
                .into_iter()
                .enumerate()
                .map(|(index, g)| {
                    self.1
                        .get(&(index as u64))
                        .map(|x| (*x).to_owned())
                        .unwrap_or_else(|| g.resolved.qualified_name())
                })
                .collect::<Vec<_>>();

            Delimited(gen.as_slice(), ", ").fmt(f)?;
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
    use phf::phf_map;
    use pretty_assertions::assert_eq;

    #[test]
    fn zod_qualified_type() {
        let ty = TypeDef {
            ns: "Ns",
            ident: "abc",
            generics: Default::default(),
        };

        assert_eq!(ty.as_arg(GenericMap::empty()).to_zod_string(), "Ns.abc");
    }

    #[test]
    fn qualified_type_with_generics() {
        const TY: TypeDef = TypeDef {
            ns: "Ns",
            ident: "abc",
            generics: &[Generic::new_for::<()>("A"), Generic::new_for::<()>("B")],
        };

        assert_eq!(
            TY.as_arg(GenericMap::new(&phf_map! {
                0_u64 => "A"
            }))
            .to_zod_string(),
            "Ns.abc(A, Rs.Unit)"
        );
    }
}
