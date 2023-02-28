use zod::zod;
use zod::Namespace;

#[derive(zod)]
#[zod(namespace = "Ns")]
enum MyEnum {}

struct Ns {}

impl Namespace for Ns {
    const NAME: &'static str = "Ns";
}

#[derive(serde::Deserialize)]
struct NsReq;

fn main() {}
