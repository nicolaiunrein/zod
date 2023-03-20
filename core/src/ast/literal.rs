use super::{FormatTypescript, FormatZod, TypeDef};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Literal {
    pub ty: TypeDef,
    pub ts: &'static str,
    pub zod: &'static str,
}

impl FormatZod for Literal {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.zod)?;
        f.write_str("\n")
    }
}

impl FormatTypescript for Literal {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ts)?;
        f.write_str("\n")
    }
}
