use zod_derive_experiments::Zod;

struct Ns;
impl zod_core::Namespace for Ns {
    const NAME: &'static str = "MyNs";
}

#[derive(Zod)]
#[zod(namespace = "Ns")]
struct StructIo {
    pub _value: u8,
    pub _tuple: nested::TupleIo,
}

mod nested {
    use super::*;
    #[derive(Zod)]
    #[zod(namespace = "Ns")]
    pub(crate) struct TupleIo(u8, String);
}
