mod test {
    use zod_derive_experiments::Zod;
    #[derive(Zod)]
    struct Struct {
        pub value: u8,
        pub tuple: nested::Tuple,
    }

    mod nested {
        use super::*;
        #[derive(Zod)]
        pub struct Tuple(u8, String);
    }
}
