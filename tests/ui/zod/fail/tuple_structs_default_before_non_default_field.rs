use zod::Namespace;
use zod::Zod;

#[derive(Zod, serde::Deserialize)]
#[zod(namespace = "Ns")]
struct Test(#[serde(default)] String, String);

struct Ns {}

impl Namespace for Ns {
    const NAME: &'static str = "Ns";
}

fn main() {}
