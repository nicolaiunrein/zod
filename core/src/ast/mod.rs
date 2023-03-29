//! # Core building blocks of this library

#[cfg(feature = "rpc")]
pub mod rpc;

mod docs;
mod export;
mod formatter;
mod generics;
mod path;
mod schema;
mod utils;

pub use docs::*;
pub use export::*;
pub use generics::*;
pub use path::*;
pub use schema::*;

pub use formatter::*;
pub(crate) use utils::*;

#[cfg(test)]
mod test {
    #![allow(dead_code)]
    use std::collections::HashSet;

    use crate::types::Usize;
    use crate::{Namespace, RequestType, RequestTypeVisitor};

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

    impl<T1: RequestType, T2: RequestType> RequestType for MyGeneric<T1, T2> {
        const AST: Export = Export {
            docs: None,
            path: Path::new::<Ns>("MyGeneric"),
            schema: ExportSchema::Object(ObjectSchema::new(&[
                NamedField::generic("t1", "T1"),
                NamedField::generic("t2", "T2"),
            ])),
            args: &[T1::AST.inline(), T2::AST.inline()],
        };
    }

    impl<T1: RequestType, T2: RequestType> RequestTypeVisitor for MyGeneric<T1, T2> {
        fn register(ctx: &mut crate::DependencyMap)
        where
            Self: 'static,
        {
            crate::visit_req_dependencies!(ctx, T1, T2);
        }
    }

    struct MyType {
        inner_my_type: Partial<usize>,
    }

    impl RequestType for MyType {
        const AST: Export = Export {
            docs: None,
            path: Path::new::<Ns>("MyType"),
            schema: ExportSchema::Object(ObjectSchema::new(&[NamedField::new::<Partial<Usize>>(
                "my_type_inner",
            )])),
            args: &[],
        };
    }

    impl RequestTypeVisitor for MyType {
        fn register(ctx: &mut crate::DependencyMap)
        where
            Self: 'static,
        {
            crate::visit_req_dependencies!(ctx, Partial<Usize>);
        }
    }

    struct Partial<T> {
        partial_inner: MyGeneric<String, T>,
    }

    impl<T: RequestType> RequestType for Partial<T> {
        const AST: Export = Export {
            docs: None,
            path: Path::new::<Ns>("Partial"),
            schema: ExportSchema::Object(ObjectSchema::new(&[NamedField::new::<
                MyGeneric<String, T>,
            >("partial_inner")])),
            args: &[T::AST.inline()],
        };
    }

    impl<T: RequestType> RequestTypeVisitor for Partial<T> {
        fn register(ctx: &mut crate::DependencyMap)
        where
            Self: 'static,
        {
            crate::visit_req_dependencies!(ctx, MyGeneric<String, T>);
        }
    }

    #[test]
    fn nested_ok() {
        let export = <MyType>::AST;
        let expected_zod_export= "export const MyType = z.lazy(() => z.object({ my_type_inner: Ns.Partial(Rs.Usize) }));";
        let expected_ts_export = "export interface MyType { my_type_inner: Ns.Partial<Rs.Usize> }";
        assert_eq!(export.to_zod_string(), expected_zod_export);
        assert_eq!(export.to_ts_string(), expected_ts_export);
    }

    #[test]
    fn register_ok() {
        let deps = <MyType>::dependencies().resolve();
        let mut expected = HashSet::new();
        expected.insert(MyType::export());
        expected.insert(<MyGeneric<String, Usize>>::export());
        expected.insert(<Usize>::export());
        expected.insert(<String>::export());
        expected.insert(<Partial<Usize>>::export());

        assert_eq!(deps, expected);
    }

    #[test]
    fn generic_export_ok() {
        let gen = <MyGeneric<String, Usize>>::export();
        assert_eq!(gen.to_zod_string(), "export const MyGeneric = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.object({ t1: T1, t2: T2 });");
    }
}
