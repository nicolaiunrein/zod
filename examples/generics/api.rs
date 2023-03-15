use zod::{rpc, Namespace, Zod};

#[derive(serde::Serialize, serde::Deserialize, zod::Zod)]
#[zod(namespace = "Ns")]
pub struct MyType {
    value: usize,
}

#[derive(Namespace)]
struct Ns;
