use zod::zod;
use zod::Namespace;

#[derive(zod)]
#[zod(namespace = "Ns")]
struct Test;

struct Ns {}

impl Namespace for Ns {
    const NAME: &'static str = "Ns";
}

fn main() {}
