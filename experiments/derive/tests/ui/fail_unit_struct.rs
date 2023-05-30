use zod_derive_experiments::Zod;

struct Ns;
impl zod_core::Namespace for Ns {
    const NAME: &'static str = "Custom_Ns";
}

#[derive(Zod)]
#[zod(namespace = "Ns")]
struct X;

fn main() {}
