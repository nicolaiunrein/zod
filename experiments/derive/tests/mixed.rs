use zod_derive_experiments::Zod;

#[derive(Zod)]
struct StructIo {
    pub _value: u8,
    pub _tuple: nested::TupleIo,
}

mod nested {
    use super::*;
    #[derive(Zod)]
    pub struct TupleIo(u8, String);
}
