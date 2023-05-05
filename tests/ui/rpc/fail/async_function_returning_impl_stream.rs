use zod::{rpc, Namespace};

#[derive(Namespace)]
struct A;

#[rpc]
impl A {
    async fn test(&mut self) -> impl futures::Stream<Item = u8> {
        futures::stream::once(async { 0 })
    }
}

fn main() {}
