//!**_NOTE:_**  This crate is not ready for production yet!
//!
//! # Core types for the [zod](https://crates.io/zod) crate
//! The [ast] module contains types to represent the generated code in a relatively type-safe
//! manner.
//!
//! The [rpc] module contains types for generating the rpc client/server.
//!
//! ## Todo
//! - Add ast for tuple structs
//! - Add ast for enums
//! - consider where to handle serde args
//!
#![doc = document_features::document_features!()]
#![deny(unsafe_code)]

pub mod ast;
pub mod types;

#[cfg(feature = "rpc")]
pub mod rpc;

use std::{
    any::TypeId,
    collections::{BTreeMap, HashSet},
};

pub trait Register {
    fn register(_: &mut DependencyMap)
    where
        Self: 'static;

    fn dependencies() -> DependencyMap
    where
        Self: 'static,
    {
        let mut cx = DependencyMap(Default::default());
        Self::register(&mut cx);
        cx
    }
}

#[derive(Debug, PartialEq)]
pub struct DependencyMap(BTreeMap<TypeId, Option<ast::Export>>);

impl DependencyMap {
    pub fn add_self<T>(&mut self) -> bool
    where
        T: ast::Node + 'static,
    {
        let id = TypeId::of::<T>();
        self.0.insert(id, T::DEFINITION.export).is_none()
    }

    pub fn resolve(self) -> HashSet<ast::Export> {
        self.0.into_values().filter_map(|exp| exp).collect()
    }
}

#[macro_export]
macro_rules! register_dependency {
    ($ctx: ident, $($ty: ty),*) => {
        if $ctx.add_self::<Self>() {
            $(<$ty as Register>::register($ctx);)*

        }
    };

    ($ctx: ident) => {
        $ctx.add_self::<Self>();
    }
}

pub trait Namespace {
    const NAME: &'static str;
    const DOCS: Option<&'static str>;

    #[doc(hidden)]
    type UniqueMembers;
}
