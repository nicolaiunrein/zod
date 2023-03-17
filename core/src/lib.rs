//! **_NOTE:_**  This crate is not ready for production yet!
#![deny(unsafe_code)]

pub mod ast;

#[cfg(feature = "rpc")]
pub mod rpc;

mod build_ins;
use std::collections::HashSet;

pub use build_ins::*;
use rpc::codegen::RpcMember;

// #[derive(Clone, Copy)]
// pub struct Code {
// pub ns_name: &'static str,
// pub name: &'static str,
// pub type_def: &'static str,
// pub schema: &'static str,
// }

// impl Code {
// pub const fn new<T: Namespace>(
// name: &'static str,
// type_def: &'static str,
// schema: &'static str,
// ) -> Self {
// Self {
// ns_name: T::NAME,
// name,
// type_def,
// schema,
// }
// }

// pub fn is_member_of<T: Namespace + ?Sized + 'static>(&self) -> bool {
// T::NAME == self.ns_name
// }
// }

// inventory::collect!(Code);

pub trait ZodType {
    const AST: ast::ZodExport;
}

pub trait Namespace {
    const NAME: &'static str;
    const DOCS: Option<&'static str>;
    fn code_members() -> Vec<ast::ZodExport>
    where
        Self: 'static,
    {
        let all = inventory::iter::<ast::ZodExport>();
        let mut own: Vec<_> = all
            .filter(|code| code.is_member_of::<Self>())
            .map(|code| *code)
            .collect();

        own.sort_by_key(|code| code.name());
        own
    }

    fn rpc_members() -> Vec<RpcMember>
    where
        Self: 'static,
    {
        let all = inventory::iter::<RpcMember>();
        let mut own: Vec<_> = all
            .filter(|rpc| rpc.ns_name() == Self::NAME)
            .map(|rpc| *rpc)
            .collect();

        own.sort_by_key(|rpc| rpc.name());
        own
    }

    fn generate() -> String
    where
        Self: 'static,
    {
        //TODO refactor this code
        let mut seen = HashSet::new();
        let mut out = String::from("export namespace ");
        out.push_str(Self::NAME);
        out.push_str(" { \n");
        for member in Self::code_members() {
            if seen.get(member.name()).is_none() {
                out.push_str(&format!("{}\n", member));
                seen.insert(member.name());
            }

            out.push_str("\n");
        }

        let mut seen = HashSet::new();
        for member in Self::rpc_members() {
            if seen.get(&member.decl()).is_none() {
                out.push_str(&format!(
                    "{}\n",
                    member
                        .decl()
                        .lines()
                        .map(|line| format!("  {line}\n"))
                        .collect::<String>()
                ));
                seen.insert(member.decl());
            }

            out.push_str("\n");
        }
        out.push_str("}");
        out
    }
}
