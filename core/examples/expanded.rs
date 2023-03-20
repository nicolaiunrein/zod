#![allow(dead_code)]

use pretty_assertions::assert_eq;
use std::sync::Arc;
use zod_core::ast::Generic;

use zod_core::{
    ast::{self, FormatZod, MaybeFlatField, NamedField, StructFields, ZodDefinition, ZodExport},
    rpc::codegen::RpcNamespace,
    DependencyRegistration, Namespace, ZodType,
};

struct MyType {
    inner: Vec<Arc<(String, usize)>>,
}

struct MyType2<T> {
    inner: T,
}

struct MyType3 {
    inner: MyType2<MyType>,
}

impl ZodType for MyType {
    const AST: ZodExport = ZodExport {
        docs: None,
        def: ZodDefinition::Struct(ast::Struct {
            ty: ast::TypeDef {
                ns: "Ns",
                ident: "MyType",
                generics: &[],
            },
            fields: StructFields::Named(&[MaybeFlatField::Named(NamedField {
                optional: false,
                name: "inner",
                value: &<Vec<Arc<(Option<String>, usize)>>>::AST.def,
            })]),
        }),
    };
}

impl<T: ZodType> ZodType for MyType2<T> {
    const AST: ZodExport = ZodExport {
        docs: None,
        def: ZodDefinition::Struct(ast::Struct {
            ty: ast::TypeDef {
                ns: "Ns",
                ident: "MyType2",
                generics: &[Generic::new_for::<T>("T")],
            },
            fields: StructFields::Named(&[MaybeFlatField::Named(NamedField {
                optional: false,
                name: "inner",
                value: &<T>::AST.def,
            })]),
        }),
    };
}

impl ZodType for MyType3 {
    const AST: ZodExport = ZodExport {
        docs: None,
        def: ZodDefinition::Struct(ast::Struct {
            ty: ast::TypeDef {
                ns: "Ns",
                ident: "MyType3",
                generics: &[],
            },
            fields: StructFields::Named(&[MaybeFlatField::Named(NamedField {
                optional: false,
                name: "inner",
                value: &<MyType2<MyType>>::AST.def,
            })]),
        }),
    };
}

impl DependencyRegistration for MyType {
    fn register_dependencies(cx: &mut zod_core::DependencyMap)
    where
        Self: 'static,
    {
        if cx.add::<Self>() {
            <Vec<Arc<(String, usize)>>>::register_dependencies(cx);
        }
    }
}

impl<T: ZodType> DependencyRegistration for MyType2<T> {
    fn register_dependencies(cx: &mut zod_core::DependencyMap)
    where
        Self: 'static,
    {
        if cx.add::<Self>() {
            <T>::register_dependencies(cx);
        }
    }
}

impl DependencyRegistration for MyType3 {
    fn register_dependencies(cx: &mut zod_core::DependencyMap)
    where
        Self: 'static,
    {
        if cx.add::<Self>() {
            <MyType2<MyType>>::register_dependencies(cx);
        }
    }
}

// generated to avoid duplicate type names
impl MyNamespaceItemRegistry {
    #[allow(non_upper_case_globals)]
    #[allow(dead_code)]
    const MyType: () = {};

    // #[allow(non_upper_case_globals)]
    // const MyType: () = {};
}

struct MyNamespace;
struct MyNamespaceItemRegistry;

#[derive(serde::Deserialize)]
enum MyNamespaceRequest {}

impl Namespace for MyNamespace {
    const NAME: &'static str = "Ns";
    const DOCS: Option<&'static str> = Some("My Namespace Docs");

    type Registry = MyNamespaceItemRegistry;
}

impl RpcNamespace for MyNamespace {
    type Req = MyNamespaceRequest;
}

impl DependencyRegistration for MyNamespace {
    fn register_dependencies(cx: &mut zod_core::DependencyMap)
    where
        Self: 'static,
    {
        // repeat for all types
        MyType::register_dependencies(cx);
    }
}

struct MyBackend {}

fn main() {
    let expected = "export const MyType = z.lazy(() => z.object({inner: Rs.Vec(Rs.Tuple2(Rs.Option(Rs.String), Rs.Usize))}));";

    assert_eq!(MyType::AST.to_zod_string(), expected);

    let out = <MyType3>::dependencies()
        .resolve()
        .into_iter()
        .map(|export| export.to_string())
        .collect::<String>();

    println!("{out}")
}
