use zod::{rpc, Namespace};

#[derive(Namespace)]
struct A;

#[rpc]
impl A {
    async fn test() {}
}

fn main() {}
