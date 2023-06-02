mod test {
    struct Ns;
    impl zod_core::Namespace for Ns {
        const NAME: &'static str = "MyNs";
    }

    #[derive(zod_derive::Zod, serde::Serialize)]
    #[zod(namespace = "Ns")]
    struct Struct(#[serde(default)] u8, bool);
}

fn main() {}
