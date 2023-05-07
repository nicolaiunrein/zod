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

use ast::{Docs, Export, Ref};

/// Trait for dependency registration
/// Each implementor should recursively call register on all its dependencies (ie. fields in a
/// struct).
///
/// # Example
/// ## using the helper macro
/// ```
/// # use zod_core::{
/// #   RequestType,
/// #   RequestTypeVisitor,
/// #   types,
/// #   ast,
/// #   DependencyMap,
/// #   visit_req_dependencies,
/// #   ast::Export,
/// #   types::Rs,
/// #   ast::ObjectSchema,
/// #   ast::ExportSchema,
/// #   ast::Path,
/// #   ast::Ref
/// # };
/// #
/// # struct MyType<T: RequestType> {
/// #     field1: Option<types::Usize>,
/// #     field2: String,
/// #     field3: T
/// # }
/// #
/// # impl<T: RequestType> RequestType for MyType<T> {
/// #     const EXPORT: Export = Export {
/// #        docs: None,
/// #        path: Path::new::<Rs>("MyType"),
/// #        schema: ExportSchema::Object(ObjectSchema::new(&[]))
/// #     };
/// #     const ARGS: &'static [Ref] = &[];
/// # }
///
/// impl<T: RequestType> RequestTypeVisitor for MyType<T> {
///     fn register(ctx: &mut DependencyMap)
///     where
///         Self: 'static,
///     {
///         visit_req_dependencies!(ctx, Option<types::Usize>, String, T);
///     }
/// }
/// ```
/// ## manual impl
///
/// **NOTE**: manual implementation should be avoided if possible because one could easily implement it
/// incorrectly. In the commented case only direct dependencies would get registered breaking the
/// recursion.
/// ```
/// # use zod_core::{
/// #   RequestType,
/// #   RequestTypeVisitor,
/// #   types,
/// #   ast,
/// #   DependencyMap,
/// #   visit_req_dependencies,
/// #   ast::Export,
/// #   types::Rs,
/// #   ast::ObjectSchema,
/// #   ast::ExportSchema,
/// #   ast::Path,
/// #   ast::Ref
/// # };
/// #
/// # struct MyType<T: RequestType> {
/// #     field1: Option<types::Usize>,
/// #     field2: String,
/// #     field3: T
/// # }
/// #
/// # impl<T: RequestType> RequestType for MyType<T> {
/// #     const EXPORT: Export = Export {
/// #        docs: None,
/// #        path: Path::new::<Rs>("MyType"),
/// #        schema: ExportSchema::Object(ObjectSchema::new(&[]))
/// #     };
/// #     const ARGS: &'static [Ref] = &[];
/// # }
/// #
/// impl<T: RequestType> RequestTypeVisitor for MyType<T> {
///     fn register(ctx: &mut DependencyMap)
///     where
///         Self: 'static,
///     {
///         if ctx.add_self_as_req::<Self>() {
///             <Option<types::Usize>>::register(ctx);
///             <String>::register(ctx);
///             <T>::register(ctx);
///         }
///
///         // THIS WOULD GO WRONG:
///         //
///         // if ctx.add_self_as_req::<Self>() {
///         //     ctx.add_self_as_req::<Option<types::Usize>>();
///         //     ctx.add_self_as_req::<String>>();
///         //     ctx.add_self_as_req::<T>>();
///         // }
///     }
/// }
/// ```
///

pub trait RequestType: RequestTypeVisitor {
    const EXPORT: Export;
    const ARGS: &'static [Ref];

    fn export() -> Export {
        Self::EXPORT
    }

    fn docs() -> Option<Docs> {
        Self::EXPORT.docs
    }
}

pub trait ResponseType: ResponseTypeVisitor {
    const EXPORT: Export;
    const ARGS: &'static [Ref];

    fn export() -> Export {
        Self::EXPORT
    }

    fn docs() -> Option<Docs> {
        Self::EXPORT.docs
    }
}

pub trait RequestTypeVisitor {
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

pub trait ResponseTypeVisitor {
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
pub struct DependencyMap(BTreeMap<TypeId, ast::Export>);

impl DependencyMap {
    pub fn add_self_as_req<T>(&mut self) -> bool
    where
        T: RequestType + 'static,
    {
        let id = TypeId::of::<T>();
        self.0.insert(id, T::EXPORT).is_none()
    }

    pub fn add_self_as_res<T>(&mut self) -> bool
    where
        T: ResponseType + 'static,
    {
        let id = TypeId::of::<T>();
        self.0.insert(id, T::EXPORT).is_none()
    }

    pub fn resolve(self) -> HashSet<ast::Export> {
        self.0.into_values().collect()
    }

    pub fn add_stream_output<F, S, T>(&mut self, _: F)
    where
        F: Fn() -> S,
        S: futures::Stream<Item = T>,
        T: ResponseType + 'static,
    {
        Self::add_self_as_res::<T>(self);
    }
}

impl std::ops::Deref for DependencyMap {
    type Target = BTreeMap<TypeId, ast::Export>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// helper macro to generate the implementation of the [RequestTypeVisitor::register] method
#[macro_export]
macro_rules! visit_req_dependencies {
    ($ctx: ident, $($ty: ty),*) => {
        if $ctx.add_self_as_req::<Self>() {
            $(<$ty as $crate::RequestTypeVisitor>::register($ctx);)*
        }
    };

    ($ctx: ident) => {
        $ctx.add_self_as_req::<Self>();
    }
}

/// helper macro to generate the implementation of the [ResponseTypeVisitor::register] method
#[macro_export]
macro_rules! visit_res_dependencies {
    ($ctx: ident, $($ty: ty),*) => {
        if $ctx.add_self_as_res::<Self>() {
            $(<$ty as $crate::ResponseTypeVisitor>::register($ctx);)*
        }
    };

    ($ctx: ident) => {
        $ctx.add_self_as_res::<Self>();
    }
}

pub trait Namespace {
    const NAME: &'static str;
    const DOCS: Option<Docs>;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn register_stream_output_ok() {
        let mut map = DependencyMap(Default::default());
        struct X;

        impl X {
            fn test(&mut self) -> impl futures::Stream<Item = String> {
                futures::stream::empty()
            }
        }

        #[allow(unreachable_code)]
        map.add_stream_output(|| X::test(todo!()));
    }
}
