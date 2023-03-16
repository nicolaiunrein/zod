use super::{FormatTypescript, FormatZod, GenericTypeParams};

#[derive(Clone, Copy, Debug)]
pub struct Type {
    pub ident: &'static str,
    pub generics: GenericTypeParams,
}

impl FormatZod for Type {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ident)?;
        self.generics.fmt_zod(f)?;
        Ok(())
    }
}

impl FormatTypescript for Type {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_zod(f)
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
            generics: GenericTypeParams::default(),
        };

        assert_eq!(ty.to_zod_string(), "abc");
    }

    #[test]
    fn zod_type_with_generics() {
        let ty = Type {
            ident: "abc",
            generics: GenericTypeParams(&[
                Generic::Regular { ident: "A" },
                Generic::Regular { ident: "B" },
            ]),
        };

        assert_eq!(ty.to_zod_string(), "abc<A, B>");
    }
}
