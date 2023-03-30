use zod::{rpc, Namespace};

#[derive(Namespace)]
struct A;

#[rpc::namespace]
impl A {
    async fn test(&mut self) -> impl futures::Stream<Item = usize> {
        futures::stream::once(async { 0 })
    }
}

fn main() {}
