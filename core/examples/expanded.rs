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

impl ZodType for MyType {
    const AST: ZodExport = ZodExport {
        docs: Some("My Docs"),
        def: ZodDefinition::Struct(ast::Struct {
            ty: ast::QualifiedType {
                ns: "Ns",
                ident: "MyType",
                generics: &[Generic::new_for::<()>("T")],
            },
            fields: StructFields::Named(&[MaybeFlatField::Named(NamedField {
                optional: false,
                name: "inner",
                value: FieldValue::Inlined(<Vec<Arc<(String, usize)>>>::AST.def.ty()),
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
    let expected = "\
/**
* My Docs
*/
export const MyType = (T: z.ZodTypeAny) => z.lazy(() => z.object({inner: Rs.Vec(Rs.Tuple2(Rs.String, Rs.Usize))}));";

    assert_eq!(MyType::AST.to_zod_string(), expected);
}
