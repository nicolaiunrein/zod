use zod::{rpc, Namespace};

#[derive(Namespace)]
struct A;

#[rpc]
impl A {
    fn test(&mut self) -> impl std::iter::Iterator<Item = usize> {
        std::iter::once(0)
    }
}

fn main() {}
