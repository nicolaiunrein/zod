#![allow(dead_code)]

use pretty_assertions::assert_eq;
use std::sync::Arc;
use zod_core::ast::Generic;

use zod_core::{
    ast::{
        self, FieldValue, FormatZod, MaybeFlatField, NamedField, StructFields, ZodDefinition,
        ZodExport,
    },
    rpc::codegen::RpcNamespace,
    DependencyRegistration, Namespace, ZodType,
};

struct MyType {
    inner: Vec<Arc<(String, usize)>>,
}

struct MyType2<T1, T2> {
    inner1: T1,
    inner2: T2,
}

struct MyType3<T> {
    inner: MyType2<T, String>,
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
                value: FieldValue::Inlined(<Vec<Arc<(String, usize)>>>::AST.def.ty()),
            })]),
        }),
    };
}

impl<T1: ZodType, T2: ZodType> ZodType for MyType2<T1, T2> {
    const AST: ZodExport = ZodExport {
        docs: None,
        def: ZodDefinition::Struct(ast::Struct {
            ty: ast::TypeDef {
                ns: "Ns",
                ident: "MyType2",
                generics: &[Generic::new_for::<T1>("T1"), Generic::new_for::<T2>("T2")],
            },
            fields: StructFields::Named(&[
                MaybeFlatField::Named(NamedField {
                    optional: false,
                    name: "inner1",
                    value: FieldValue::Generic("T1"),
                }),
                MaybeFlatField::Named(NamedField {
                    optional: false,
                    name: "inner2",
                    value: FieldValue::Generic("T2"),
                }),
            ]),
        }),
    };
}

impl<T: ZodType> ZodType for MyType3<T> {
    const AST: ZodExport = ZodExport {
        docs: None,
        def: ZodDefinition::Struct(ast::Struct {
            ty: ast::TypeDef {
                ns: "Ns",
                ident: "MyType3",
                generics: &[Generic::new_for::<T>("T")],
            },
            fields: StructFields::Named(&[MaybeFlatField::Named(NamedField {
                optional: false,
                name: "inner",
                value: FieldValue::Qualified(<MyType2<T, String>>::AST.def.ty()),
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

impl<T1: ZodType, T2: ZodType> DependencyRegistration for MyType2<T1, T2> {
    fn register_dependencies(cx: &mut zod_core::DependencyMap)
    where
        Self: 'static,
    {
        if cx.add::<Self>() {
            <T1>::register_dependencies(cx);
            <T2>::register_dependencies(cx);
        }
    }
}

impl<T: ZodType> DependencyRegistration for MyType3<T> {
    fn register_dependencies(cx: &mut zod_core::DependencyMap)
    where
        Self: 'static,
    {
        if cx.add::<Self>() {
            <MyType2<T, String>>::register_dependencies(cx);
        }
    }
}

// // generated to avoid duplicate type names
// impl MyNamespaceItemRegistry {
// #[allow(non_upper_case_globals)]
// #[allow(dead_code)]
// const MyType: () = {};

// // #[allow(non_upper_case_globals)]
// // const MyType: () = {};
// }

// struct MyNamespace;
// struct MyNamespaceItemRegistry;

// #[derive(serde::Deserialize)]
// enum MyNamespaceRequest {}

// impl Namespace for MyNamespace {
// const NAME: &'static str = "Ns";
// const DOCS: Option<&'static str> = Some("My Namespace Docs");

// type Registry = MyNamespaceItemRegistry;
// }

// impl RpcNamespace for MyNamespace {
// type Req = MyNamespaceRequest;
// }

// impl DependencyRegistration for MyNamespace {
// fn register_dependencies(cx: &mut zod_core::DependencyMap)
// where
// Self: 'static,
// {
// // repeat for all types
// MyType::register_dependencies(cx);
// }
// }

struct MyBackend {}

fn main() {
    let expected = "export const MyType = z.lazy(() => z.object({inner: Rs.Vec(Rs.Tuple2(Rs.String, Rs.Usize))}));";
    assert_eq!(MyType::AST.to_zod_string(), expected);

    let expected2 = "export const MyType2 = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.lazy(() => z.object({inner1: T1, inner2: T2}));";
    assert_eq!(MyType2::<(), ()>::AST.to_zod_string(), expected2);

    let expected3 = "export const MyType3 = (T: z.ZodTypeAny) => z.lazy(() => z.object({inner: Ns.MyType2(T, Rs.String)}));";
    assert_eq!(MyType3::<()>::AST.to_zod_string(), expected3);

    let out = <MyType3<()>>::dependencies()
        .resolve()
        .into_iter()
        .map(|export| export.to_string())
        .collect::<String>();

    println!("{out}")
}
