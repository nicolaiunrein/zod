use zod::{Namespace, Zod};

#[derive(serde::Serialize, serde::Deserialize, Zod, Default, Debug)]
#[zod(namespace = "Ns")]
pub struct MyType {
    value2: usize,
}

#[derive(serde::Serialize, serde::Deserialize, Zod, Debug)]
#[zod(namespace = "Ns")]
pub struct MyType2 {
    value: u16,
    // #[serde(flatten)]
    nested: MyType,
}

#[derive(serde::Serialize, serde::Deserialize, Zod, Debug)]
#[zod(namespace = "Ns")]
pub struct MyType3(usize, usize);

#[derive(Namespace)]
pub struct Ns;
