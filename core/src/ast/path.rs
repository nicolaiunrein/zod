use std::fmt::Display;

use crate::Namespace;

/// Qualified name of an [Export](crate::Export)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Path {
    ns: &'static str,
    name: &'static str,
    pub(crate) generic: Option<usize>,
}

impl Path {
    pub const fn new<T: Namespace>(name: &'static str) -> Self {
        Self {
            ns: T::NAME,
            name,
            generic: None,
        }
    }

    pub const fn generic(index: usize) -> Self {
        Self {
            ns: "",
            name: "",
            generic: Some(index),
        }
    }

    pub const fn ns(&self) -> &'static str {
        self.ns
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.generic {
            Some(index) => f.write_fmt(format_args!("___{}__", index))?,
            None => {
                f.write_str(self.ns)?;
                f.write_str(".")?;
                f.write_str(self.name)?;
            }
        }
        Ok(())
    }
}
