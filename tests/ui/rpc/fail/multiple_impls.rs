use zod::{rpc, Namespace};

#[derive(Namespace)]
struct A;

#[rpc]
impl A {}

#[rpc]
impl A {}

fn main() {}
