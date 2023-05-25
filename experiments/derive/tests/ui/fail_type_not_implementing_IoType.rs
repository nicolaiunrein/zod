mod test {
    struct X;

    #[derive(zod_derive_experiments::Zod)]
    struct Struct {
        pub value: u8,
        pub tuple: X,
    }

    #[derive(zod_derive_experiments::Zod)]
    struct Tuple(u8, X);
}

fn main() {}
