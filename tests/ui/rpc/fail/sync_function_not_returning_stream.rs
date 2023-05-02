use zod::{rpc, Namespace};

#[derive(Namespace)]
struct A;

#[rpc]
impl A {
    fn test(&mut self) -> String {
        String::new()
    }
}

fn main() {}
