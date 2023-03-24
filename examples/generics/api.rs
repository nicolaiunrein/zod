use zod::{types::Usize, Namespace, Node};

#[derive(serde::Serialize, serde::Deserialize, Node, Default, Debug)]
#[zod(namespace = "Ns")]
pub struct MyType {
    value2: Usize,
}

#[derive(serde::Serialize, serde::Deserialize, Node, Debug)]
#[zod(namespace = "Ns")]
pub struct MyType2 {
    value: u16,
    // #[serde(flatten)]
    nested: MyType,
}

#[derive(serde::Serialize, serde::Deserialize, Node, Debug)]
#[zod(namespace = "Ns")]
pub struct MyType3(Usize, Usize);

#[derive(Namespace)]
pub struct Ns;
