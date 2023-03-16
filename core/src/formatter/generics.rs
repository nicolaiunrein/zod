use super::{FormatTypescript, FormatZod};

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

impl FormatTypescript for Generic {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Regular { ident } => f.write_str(ident),
        }
    }
}

// #[derive(Clone, Copy, Debug, Default)]
// pub struct Generics(pub &'static [Generic]);

// impl FormatZod for Generics {
// fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// let mut iter = self.0.iter().peekable();

// while let Some(item) = iter.next() {
// item.fmt_zod(f)?;
// if iter.peek().is_some() {
// f.write_str(", ")?;
// }
// }

// Ok(())
// }
// }

// impl FormatTypescript for Generics {
// fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// let mut iter = self.0.iter().peekable();

// while let Some(item) = iter.next() {
// item.fmt_ts(f)?;
// if iter.peek().is_some() {
// f.write_str(", ")?;
// }
// }

// Ok(())
// }
// }

// #[cfg(test)]
// mod test {
// use super::*;
// use crate::formatter::ZodTypeAny;
// use pretty_assertions::assert_eq;

// #[test]
// fn zod_generic() {
// let gen = Generic::Regular { ident: "T" };
// assert_eq!(gen.to_zod_string(), "T");
// }

// #[test]
// fn zod_generic_function_params() {
// let gens = &[
// Generic::Regular { ident: "A" },
// Generic::Regular { ident: "B" },
// ];

// assert_eq!(
// Generics(gens).to_zod_string(),
// format!("(A: {ZodTypeAny}, B: {ZodTypeAny})")
// );

// assert_eq!(
// Generics(&gens[0..1]).to_zod_string(),
// format!("(A: {ZodTypeAny})")
// );

// assert_eq!(Generics(&[]).to_zod_string(), "()")
// }
// }
