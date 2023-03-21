//! **_NOTE:_**  This crate is not ready for production yet!
#![deny(unsafe_code)]

pub mod ast;

#[cfg(feature = "rpc")]
pub mod rpc;

use std::{
    any::TypeId,
    collections::{BTreeMap, HashSet},
};

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
pub struct DependencyMap(BTreeMap<TypeId, Option<ast::Export>>);

impl DependencyMap {
    pub fn add<T>(&mut self) -> bool
    where
        T: ast::Node + 'static,
    {
        let id = TypeId::of::<T>();
        self.0.insert(id, T::export()).is_none()
    }

    pub fn resolve(self) -> HashSet<ast::Export> {
        self.0.into_values().filter_map(|exp| exp).collect()
    }
}

pub trait Namespace {
    const NAME: &'static str;
    const DOCS: Option<&'static str>;
}
