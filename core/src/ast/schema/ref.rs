use crate::ast::{Compiler, Delimited, Path};
use crate::{RequestType, ResponseType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ref {
    path: Path,
    args: &'static [Ref],
}

impl Ref {
    pub const fn new_req<T: RequestType>() -> Self {
        let path = T::EXPORT.path;
        let args = T::ARGS;

        Self { path, args }
    }

    pub const fn new_res<T: ResponseType>() -> Self {
        let path = T::EXPORT.path;
        let args = T::ARGS;

        Self { path, args }
    }
}

impl Compiler for Ref {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.path, f)?;
        if !self.args.is_empty() {
            f.write_str("(")?;
            self.args
                .iter()
                .comma_separated(f, |f, arg| arg.fmt_zod(f))?;

            f.write_str(")")?;
        }

        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.path, f)?;
        if !self.args.is_empty() {
            f.write_str("<")?;
            self.args
                .iter()
                .comma_separated(f, |f, arg| arg.fmt_ts(f))?;
            f.write_str(">")?;
        }
        Ok(())
    }
}
