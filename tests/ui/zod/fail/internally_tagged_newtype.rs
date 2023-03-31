#[derive(serde::Serialize, serde::Deserialize, zod::RequestType)]
#[zod(namespace = "Ns")]
#[serde(tag = "type")]
enum Test {
    Inner(u8),
}

#[derive(zod::Namespace)]
struct Ns;

fn main() {}
