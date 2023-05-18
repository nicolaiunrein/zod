use crate::ast::{Compiler, Delimited, GenericArgument};

use super::Exported;

// TODO: RawSchema does not work with transforming generics

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RawSchema {
    pub args: &'static [GenericArgument],
    pub ts: &'static str,
    pub zod: &'static str,
}

impl RawSchema {
    pub const fn export(self, name: &'static str) -> Exported<Self> {
        Exported::new(name, self)
    }
}

impl Compiler for Exported<RawSchema> {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("const {} = ", self.name))?;

        if !self.schema.args.is_empty() {
            f.write_str("(")?;
            self.schema
                .args
                .iter()
                .filter(|arg| !arg.is_assign())
                .comma_separated(f, |f, arg| arg.fmt_zod(f))?;

            f.write_str(") => ")?;
        }
        f.write_str(self.schema.zod)?;
        f.write_str(";")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("type {}", self.name))?;

        if !self.schema.args.is_empty() {
            f.write_str("<")?;
            self.schema
                .args
                .iter()
                .comma_separated(f, |f, arg| arg.fmt_ts(f))?;
            f.write_str(">")?;
        }
        f.write_str(" = ")?;
        f.write_str(self.schema.ts)?;
        f.write_str(";")?;
        Ok(())
    }
}
