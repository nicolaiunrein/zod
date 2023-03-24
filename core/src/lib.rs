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

use ast::Docs;

/// Trait for dependency registration
/// Each implementor should recursively call register on all its dependencies (ie. fields in a
/// struct).
///
/// # Example
/// ## using the helper macro
/// ```
/// # use zod_core::{ast::Node, ast::InlineSchema, Register, ast::Definition, types, ast,
/// DependencyMap, register_dependencies};
/// #
/// # struct MyType<T: Node> {
/// #     field1: Option<types::Usize>,
/// #     field2: String,
/// #     field3: T
/// # }
/// #
/// # impl<T: Node> Node for MyType<T> {
/// #     const DEFINITION: ast::Definition =
/// #         Definition::Inlined(InlineSchema::Tuple(ast::TupleSchema::new(&[])));
/// # }
/// #
/// impl<T: Node> Register for MyType<T> {
///     fn register(ctx: &mut DependencyMap)
///     where
///         Self: 'static,
///     {
///         register_dependencies!(ctx, Option<types::Usize>, String, T);
///     }
/// }
/// ```
/// ## manual impl
///
/// **NOTE**: manual implementation should be avoided if possible because one could easily implement it
/// incorrectly. In the commented case only direct dependencies would get registered breaking the
/// recursion.
/// ```
/// # use zod_core::{ast::Node, ast::InlineSchema, Register, ast::Definition, types, ast,
/// DependencyMap};
/// #
/// # struct MyType<T: Node> {
/// #     field1: Option<types::Usize>,
/// #     field2: String,
/// #     field3: T
/// # }
/// #
/// # impl<T: Node> Node for MyType<T> {
/// #     const DEFINITION: ast::Definition =
/// #         Definition::Inlined(InlineSchema::Tuple(ast::TupleSchema::new(&[])));
/// # }
/// #
/// impl<T: Node> Register for MyType<T> {
///     fn register(ctx: &mut DependencyMap)
///     where
///         Self: 'static,
///     {
///         if ctx.add_self::<Self>() {
///             <Option<types::Usize>>::register(ctx);
///             <String>::register(ctx);
///             <T>::register(ctx);
///         }
///
///         // THIS WOULD GO WRONG:
///         //
///         // if ctx.add_self::<Self>() {
///         //     ctx.add_self::<Option<types::Usize>>();
///         //     ctx.add_self::<String>>();
///         //     ctx.add_self::<T>>();
///         // }
///     }
/// }
/// ```
///

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
        self.0.insert(id, T::DEFINITION.export()).is_none()
    }

    pub fn resolve(self) -> HashSet<ast::Export> {
        self.0.into_values().filter_map(|exp| exp).collect()
    }
}

/// helper macro to generate the implementation of the [Register::register] method
#[macro_export]
macro_rules! register_dependencies {
    ($ctx: ident, $($ty: ty),*) => {
        if $ctx.add_self::<Self>() {
            $(<$ty as $crate::Register>::register($ctx);)*
        }
    };

    ($ctx: ident) => {
        $ctx.add_self::<Self>();
    }
}

pub trait Namespace {
    const NAME: &'static str;
    const DOCS: Option<Docs>;
}
