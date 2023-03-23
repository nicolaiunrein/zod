use std::fmt::Display;

use crate::Namespace;

/// Qualified name of an exported [Node](crate::ast::Node)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Path {
    ns: &'static str,
    name: &'static str,
}

impl Path {
    pub const fn new<T: Namespace>(name: &'static str) -> Self {
        Self { ns: T::NAME, name }
    }

    pub const fn ns(&self) -> &'static str {
        &self.ns
    }

    pub const fn name(&self) -> &'static str {
        &self.name
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ns)?;
        f.write_str(".")?;
        f.write_str(self.name)?;
        Ok(())
    }
}