use zod::Namespace;
use zod::Zod;

#[derive(Zod)]
#[zod(namespace = "Ns")]
struct Test;

struct Ns {}

impl Namespace for Ns {
    const NAME: &'static str = "Ns";
}

fn main() {}
