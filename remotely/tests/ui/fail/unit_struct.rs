use remotely::zod;
use remotely_core::codegen::namespace::Namespace;

#[derive(zod)]
#[zod(namespace = "Ns")]
struct Test;

struct Ns {}

impl Namespace for Ns {
    const NAME: &'static str = "Ns";
    type Req = NsReq;
}

#[derive(serde::Deserialize)]
struct NsReq;

fn main() {}
