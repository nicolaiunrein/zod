use zod::{rpc, Namespace};

#[derive(Namespace)]
struct A;

#[rpc::namespace]
impl A {
    fn test(&mut self) -> String {
        String::new()
    }
}

fn main() {}
