use crate::ast::{Delimited, Formatter, GenericArgument};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RawSchema {
    pub args: &'static [GenericArgument],
    pub ts: &'static str,
    pub zod: &'static str,
}

impl Formatter for (&'static str, RawSchema) {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.0;
        let schema = self.1;

        f.write_fmt(format_args!("const {name} = "))?;

        if !schema.args.is_empty() {
            f.write_str("(")?;
            schema
                .args
                .iter()
                .filter(|arg| !arg.is_assign())
                .comma_separated(f, |f, arg| arg.fmt_zod(f))?;

            f.write_str(") => ")?;
        }
        f.write_str(schema.zod)?;
        f.write_str(";")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.0;
        let schema = self.1;
        f.write_fmt(format_args!("type {name}"))?;

        if !schema.args.is_empty() {
            f.write_str("<")?;
            schema
                .args
                .iter()
                .comma_separated(f, |f, arg| arg.fmt_ts(f))?;
            f.write_str(">")?;
        }
        f.write_str(" = ")?;
        f.write_str(schema.ts)?;
        f.write_str(";")?;
        Ok(())
    }
}
