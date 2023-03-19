use std::sync::Arc;

use zod_core::{
    ast::{
        self, FieldValue, MaybeFlatField, NamedField, QualifiedType, StructFields, ZodDefinition,
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
            ns: "Ns",
            ty: ast::Type {
                ident: "MyType",
                generics: &[],
            },
            fields: StructFields::Named(&[MaybeFlatField::Named(NamedField {
                optional: false,
                name: "inner",
                // value: <Vec<Arc<(String, usize)>> as ZodType>::AST.ty(),
                value: todo!(),
            })]),
        }),
    };

    fn inline_zod() -> String {
        format!("{}.{}", Self::AST.ns(), Self::AST.name())
    }
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
fn main() {}
