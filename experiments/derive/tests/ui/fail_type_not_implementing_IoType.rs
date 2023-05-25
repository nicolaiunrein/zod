mod test {
    struct Ns;
    impl zod_core::Namespace for Ns {
        const NAME: &'static str = "MyNs";
    }

    struct X;

    #[derive(zod_derive_experiments::Zod)]
    #[zod(namespace = "Ns")]
    struct Struct {
        pub value: u8,
        pub tuple: X,
    }

    #[derive(zod_derive_experiments::Zod)]
    #[zod(namespace = "Ns")]
    struct Tuple(u8, X);
}

fn main() {}
