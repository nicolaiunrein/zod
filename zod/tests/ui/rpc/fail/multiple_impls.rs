use zod::{rpc, Namespace};

#[derive(Namespace)]
struct A;

#[rpc::namespace]
impl A {}

#[rpc::namespace]
impl A {}

fn main() {}
