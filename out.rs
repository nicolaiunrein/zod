#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use zod::types::Usize;
#[zod(namespace = "Ns")]
struct Test {
    s: String,
    num: Usize,
}
impl ::zod::core::RequestType for Test {
    const ARGS: &'static [::zod::core::ast::Ref] = &[];
    const EXPORT: ::zod::core::ast::Export = ::zod::core::ast::Export {
        docs: None,
        path: ::zod::core::ast::Path::new::<Ns>("Test"),
        schema: ::zod::core::ast::ExportSchema::Object(::zod::core::ast::ObjectSchema::new(&[])),
    };
}
impl ::zod::core::RequestTypeVisitor for Test {
    fn register(ctx: &mut ::zod::core::DependencyMap)
    where
        Self: 'static,
    {
        if ctx.add_self_as_req::<Self>() {
            <String as ::zod_core::RequestTypeVisitor>::register(ctx);
            <Usize as ::zod_core::RequestTypeVisitor>::register(ctx);
        }
    }
}
struct Ns;
impl ::zod::core::Namespace for Ns {
    const NAME: &'static str = "Ns";
    const DOCS: Option<::zod::core::ast::Docs> = None;
}
#[rustc_main]
pub fn main() -> () {
    extern crate test;
    test::test_main_static(&[])
}
