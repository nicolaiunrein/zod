use std::{fmt::Display, ops::Deref};

use super::{FormatZod, ZodTypeAny};

pub use function_params::*;
pub use type_params::*;

#[derive(Clone, Copy, Debug)]
pub enum Generic {
    Regular { ident: &'static str },
}

impl FormatZod for Generic {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Regular { ident } => f.write_str(ident),
        }
    }
}

mod function_params {
    use super::*;

    #[derive(Clone, Copy, Debug, Default)]
    pub struct GenericFunctionParams(pub &'static [Generic]);

    impl From<GenericTypeParams> for GenericFunctionParams {
        fn from(value: GenericTypeParams) -> Self {
            Self(value.0)
        }
    }

    impl Deref for GenericFunctionParams {
        type Target = &'static [Generic];

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl FormatZod for GenericFunctionParams {
        fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("(")?;

            let mut iter = self.0.iter().peekable();

            while let Some(gen) = iter.next() {
                gen.fmt_zod(f)?;
                f.write_str(": ")?;
                ZodTypeAny.fmt(f)?;
                if iter.peek().is_some() {
                    f.write_str(", ")?;
                }
            }

            f.write_str(")")?;
            Ok(())
        }
    }
}

mod type_params {
    use super::*;

    #[derive(Clone, Copy, Debug, Default)]
    pub struct GenericTypeParams(pub &'static [Generic]);

    impl From<GenericFunctionParams> for GenericTypeParams {
        fn from(value: GenericFunctionParams) -> Self {
            Self(value.0)
        }
    }

    impl Deref for GenericTypeParams {
        type Target = &'static [Generic];

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl FormatZod for GenericTypeParams {
        fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if !self.0.is_empty() {
                f.write_str("<")?;

                let mut iter = self.0.iter().peekable();

                while let Some(gen) = iter.next() {
                    gen.fmt_zod(f)?;
                    if iter.peek().is_some() {
                        f.write_str(", ")?;
                    }
                }

                f.write_str(">")?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn zod_generic() {
        let gen = Generic::Regular { ident: "T" };
        assert_eq!(gen.to_zod_string(), "T");
    }

    #[test]
    fn zod_generic_function_params() {
        let gens = &[
            Generic::Regular { ident: "A" },
            Generic::Regular { ident: "B" },
        ];

        assert_eq!(
            GenericFunctionParams(gens).to_zod_string(),
            format!("(A: {ZodTypeAny}, B: {ZodTypeAny})")
        );

        assert_eq!(
            GenericFunctionParams(&gens[0..1]).to_zod_string(),
            format!("(A: {ZodTypeAny})")
        );

        assert_eq!(GenericFunctionParams(&[]).to_zod_string(), "()")
    }
}
