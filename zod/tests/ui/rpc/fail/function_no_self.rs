use zod::{rpc, Namespace};

#[derive(Namespace)]
struct A;

#[rpc::namespace]
impl A {
    async fn test() {}
}

fn main() {}
