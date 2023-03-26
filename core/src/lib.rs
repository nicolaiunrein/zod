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

use ast::{Definition, Docs, Export, InlineSchema};

/// Trait for dependency registration
/// Each implementor should recursively call register on all its dependencies (ie. fields in a
/// struct).
///
/// # Example
/// ## using the helper macro
/// ```
/// # use zod_core::{InputType, ast::InlineSchema, InputTypeVisitor, ast::Definition, types, ast,
/// DependencyMap, register_dependencies};
/// #
/// # struct MyType<T: InputType> {
/// #     field1: Option<types::Usize>,
/// #     field2: String,
/// #     field3: T
/// # }
/// #
/// # impl<T: InputType> InputType for MyType<T> {
/// #     const AST: ast::Definition =
/// #         Definition::Inlined(InlineSchema::Tuple(ast::TupleSchema::new(&[])));
/// # }
/// #
/// impl<T: InputType> InputTypeVisitor for MyType<T> {
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
/// # use zod_core::{InputType, ast::InlineSchema, InputTypeVisitor, ast::Definition, types, ast,
/// DependencyMap};
/// #
/// # struct MyType<T: InputType> {
/// #     field1: Option<types::Usize>,
/// #     field2: String,
/// #     field3: T
/// # }
/// #
/// # impl<T: InputType> InputType for MyType<T> {
/// #     const AST: ast::Definition =
/// #         Definition::Inlined(InlineSchema::Tuple(ast::TupleSchema::new(&[])));
/// # }
/// #
/// impl<T: InputType> InputTypeVisitor for MyType<T> {
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

/// ## The core trait of zod
///
/// Each and every element to be accessible for the client needs to
/// implement it.
/// This crate implements this trait for most of the relevant standard library types as well as
/// types from some third party crates. If you find yourself in need for a specific type to
/// implement this trait and you cannot implement it yourself because of the orphan rule please
/// file an issue or submit a PR. Contribution is more than welcome!
pub trait InputType: InputTypeVisitor {
    const AST: Definition;

    fn export() -> Option<Export> {
        Self::AST.export()
    }

    fn inline() -> InlineSchema {
        Self::AST.inline()
    }

    fn docs() -> Option<Docs> {
        Self::AST.docs()
    }
}

pub trait OutputType: InputTypeVisitor {
    const AST: Definition;

    fn export() -> Option<Export> {
        Self::AST.export()
    }

    fn inline() -> InlineSchema {
        Self::AST.inline()
    }

    fn docs() -> Option<Docs> {
        Self::AST.docs()
    }
}

pub trait InputTypeVisitor {
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
        T: InputType + 'static,
    {
        let id = TypeId::of::<T>();
        self.0.insert(id, T::AST.export()).is_none()
    }

    pub fn resolve(self) -> HashSet<ast::Export> {
        self.0.into_values().filter_map(|exp| exp).collect()
    }
}

/// helper macro to generate the implementation of the [InputTypeVisitor::register] method
#[macro_export]
macro_rules! register_dependencies {
    ($ctx: ident, $($ty: ty),*) => {
        if $ctx.add_self::<Self>() {
            $(<$ty as $crate::InputTypeVisitor>::register($ctx);)*
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
