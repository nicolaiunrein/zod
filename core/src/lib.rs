//! **_NOTE:_**  This crate is not ready for production yet!
#![deny(unsafe_code)]

pub mod ast;

#[cfg(feature = "rpc")]
pub mod rpc;

mod build_ins;
use std::{
    any::TypeId,
    collections::{BTreeMap, HashSet},
};

use ast::ZodExport;
pub use build_ins::*;

pub trait ZodType: DependencyRegistration {
    const AST: ast::ZodExport;

    fn inline_zod() -> String;
    // format!("{}.{}", Self::AST.ns(), Self::AST.name())
}

pub trait DependencyRegistration {
    fn register_dependencies(_: &mut DependencyMap)
    where
        Self: 'static;

    fn dependencies() -> DependencyMap
    where
        Self: 'static,
    {
        let mut cx = DependencyMap(Default::default());
        Self::register_dependencies(&mut cx);
        cx
    }
}

#[derive(Debug, PartialEq)]
pub struct DependencyMap(BTreeMap<TypeId, ZodExport>);

impl DependencyMap {
    pub fn add<T>(&mut self) -> bool
    where
        T: ZodType + 'static,
    {
        let id = TypeId::of::<T>();
        let node = T::AST;
        !self.0.insert(id, node).is_some()
    }

    pub fn resolve(self) -> HashSet<ZodExport> {
        self.0.into_iter().map(|(_, value)| value).collect()
    }
}

pub trait Namespace {
    const NAME: &'static str;
    const DOCS: Option<&'static str>;
    type Registry;

    fn generate() -> String
    where
        Self: 'static,
    {
        let mut out = String::from("export namespace ");
        out.push_str(Self::NAME);
        out.push_str(" { \n");

        //TODO ...

        out.push_str("}");
        out
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn nesting_ok() {
        assert_eq!(<Option<String>>::inline_zod(), "Rs.Option(Rs.String)");
    }
}
