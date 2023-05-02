use zod::Namespace;
use zod::RequestType;

#[derive(RequestType, serde::Deserialize)]
#[zod(namespace = "Ns")]
struct Test(#[serde(default)] String, String);

#[derive(Namespace)]
struct Ns {}

fn main() {}
