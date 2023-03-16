use super::{Delimited, FormatTypescript, FormatZod, Generic};

#[derive(Clone, Copy, Debug)]
pub struct Type {
    pub ident: &'static str,
    pub generics: &'static [Generic],
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
    use crate::formatter::Generic;

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
    fn zod_type_with_generics() {
        let ty = Type {
            ident: "abc",
            generics: &[
                Generic::Regular { ident: "A" },
                Generic::Regular { ident: "B" },
            ],
        };

        assert_eq!(ty.to_zod_string(), "abc(A, B)");
    }
}
