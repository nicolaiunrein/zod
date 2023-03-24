//! # Core building blocks of this library

#[cfg(feature = "rpc")]
pub mod rpc;

mod docs;
mod export;
mod fields;
mod formatter;
mod generics;
mod path;
mod schema;
mod utils;

pub use docs::*;
pub use export::*;
pub use fields::*;
pub use generics::*;
pub use path::*;
pub use schema::*;

pub use formatter::*;
pub(crate) use utils::*;

use crate::Register;

/// ## The core trait of zod
///
/// Each and every element to be accessible for the client needs to
/// implement it.
/// This crate implements this trait for most of the relevant standard library types as well as
/// types from some third party crates. If you find yourself in need for a specific type to
/// implement this trait and you cannot implement it yourself because of the orphan rule please
/// file an issue or submit a PR. Contribution is more than welcome!
pub trait Node: Register {
    const DEFINITION: Definition;

    fn export() -> Option<Export> {
        Self::DEFINITION.export()
    }

    fn inline() -> InlineSchema {
        Self::DEFINITION.inline()
    }
}

/// This type is the union of exported and inlined types.
///
/// **NOTE**:
/// Some types cannot be exported because their generics are only partially resolved. Therefore
/// they can only provide an InlineSchema.
///
/// # Example:
/// consider the following code:
/// ```rust
/// struct Generic<T1, T2> {
///   t1: T1,
///   t2: T2,
/// }
///
/// type Flipped<T1, T2> = Generic<T2, T1>;
///
/// // Deriving [Node] on `MyType` would generate an export like:
/// // export const MyType = (T: z.ZodTypeAny) => z.object({
/// //     ok: Ns.Generic(Rs.String, T),
/// //     flipped: Ns.Generic(T, Rs.String) // <-- oops, does not equal Flipped<T, String>
/// // })
/// struct MyType<T> {
///     ok: Generic<String, T>,
///     flipped: Flipped<T, String> // <-- equals MyGeneric<String, T>
/// }
///
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Definition {
    Exported {
        export: Export,
        args: &'static [InlineSchema],
    },

    Inlined(InlineSchema),
}

impl Definition {
    pub const fn exported(export: Export, args: &'static [InlineSchema]) -> Self {
        Self::Exported { export, args }
    }

    pub const fn inlined(schema: InlineSchema) -> Self {
        Self::Inlined(schema)
    }

    pub const fn export(self) -> Option<Export> {
        match self {
            Definition::Exported { export, .. } => Some(export),
            Definition::Inlined(_) => None,
        }
    }
    pub const fn inline(self) -> InlineSchema {
        match self {
            Definition::Exported { args, export } => InlineSchema::Ref {
                path: export.path,
                args,
            },
            Definition::Inlined(inline) => inline,
        }
    }
}

#[cfg(test)]
mod test {
    #![allow(dead_code)]
    use std::collections::HashSet;

    use crate::types::Usize;
    use crate::{Namespace, Register};

    use super::*;
    use pretty_assertions::assert_eq;

    struct Ns;
    impl Namespace for Ns {
        const NAME: &'static str = "Ns";
        const DOCS: Option<Docs> = None;
    }

    struct MyGeneric<T1, T2> {
        t1: T1,
        t2: T2,
    }

    impl<T1: Node, T2: Node> Node for MyGeneric<T1, T2> {
        const DEFINITION: Definition = Definition::exported(
            Export {
                docs: None,
                path: Path::new::<Ns>("MyGeneric"),
                schema: ExportSchema::Object(ObjectSchema::new(&[
                    NamedField::new::<T1>("t1"),
                    NamedField::new::<T2>("t2"),
                ])),
            },
            &[T1::DEFINITION.inline(), T2::DEFINITION.inline()],
        );
    }

    impl<T1: Node, T2: Node> Register for MyGeneric<T1, T2> {
        fn register(ctx: &mut crate::DependencyMap)
        where
            Self: 'static,
        {
            crate::register_dependencies!(ctx, T1, T2);
        }
    }

    struct MyType {
        inner_my_type: Partial<usize>,
    }

    impl Node for MyType {
        const DEFINITION: Definition = Definition::exported(
            Export {
                docs: None,
                path: Path::new::<Ns>("MyType"),
                schema: ExportSchema::Object(ObjectSchema::new(&[
                    NamedField::new::<Partial<Usize>>("my_type_inner"),
                ])),
            },
            &[],
        );
    }

    impl Register for MyType {
        fn register(ctx: &mut crate::DependencyMap)
        where
            Self: 'static,
        {
            crate::register_dependencies!(ctx, Partial<Usize>);
        }
    }

    struct Partial<T> {
        partial_inner: MyGeneric<String, T>,
    }

    impl<T: Node> Node for Partial<T> {
        const DEFINITION: Definition =
            Definition::inlined(InlineSchema::Object(ObjectSchema::new(&[
                NamedField::new::<MyGeneric<String, T>>("partial_inner"),
            ])));
    }

    impl<T: Node> Register for Partial<T> {
        fn register(ctx: &mut crate::DependencyMap)
        where
            Self: 'static,
        {
            crate::register_dependencies!(ctx, MyGeneric<String, T>);
        }
    }

    #[test]
    fn nested_ok() {
        let export = <MyType>::DEFINITION.export();
        let expected_zod_export= "export const MyType = z.lazy(() => z.object({ my_type_inner: z.object({ partial_inner: Ns.MyGeneric(Rs.String, Rs.Usize) }) }));";
        let expected_ts_export = "export interface MyType { my_type_inner: { partial_inner: Ns.MyGeneric<Rs.String, Rs.Usize> } }";
        assert_eq!(
            export.as_ref().unwrap().to_zod_string(),
            expected_zod_export
        );

        assert_eq!(export.as_ref().unwrap().to_ts_string(), expected_ts_export);
    }

    #[test]
    fn register_ok() {
        let deps = <MyType>::dependencies().resolve();
        let mut expected = HashSet::new();
        expected.insert(MyType::export().unwrap());
        expected.insert(<MyGeneric<String, Usize>>::export().unwrap());
        expected.insert(<Usize>::export().unwrap());
        expected.insert(<String>::export().unwrap());

        // partial does not export anything
        assert!(<Partial<crate::types::Usize>>::export().is_none());

        assert_eq!(deps, expected);
    }
}
