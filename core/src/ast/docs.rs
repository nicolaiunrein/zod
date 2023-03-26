use super::Formatter;

/// Docs to be formatted and placed above the exported type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Docs(pub &'static str);

impl AsRef<str> for Docs {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl Formatter for Docs {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("/**\n")?;
        for line in self.0.lines() {
            f.write_str(" * ")?;
            f.write_str(line)?;
            f.write_str("\n")?;
        }

        f.write_str(" */")
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_zod(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn docs_ok() {
        let expected = "\
/**
 * Hello World
 * New Line
 */";

        assert_eq!(Docs("Hello World\nNew Line").to_zod_string(), expected);
    }
}
