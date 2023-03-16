use super::{FormatTypescript, FormatZod};

#[derive(Clone, Copy, Debug)]
pub enum Generic {
    Type { ident: &'static str },
}

impl FormatZod for Generic {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type { ident } => f.write_str(ident),
        }
    }
}

impl FormatTypescript for Generic {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Type { ident } => f.write_str(ident),
        }
    }
}
