use super::{FormatTypescript, FormatZod, Type};

#[derive(Clone, Copy, Debug)]
pub struct Literal {
    pub ns: &'static str,
    pub ty: Type,
    pub ts: &'static str,
    pub zod: &'static str,
}

impl FormatZod for Literal {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.zod)
    }
}

impl FormatTypescript for Literal {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ts)
    }
}
