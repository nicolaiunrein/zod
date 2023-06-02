use pretty_assertions::assert_eq;
use zod::prelude::*;

#[test]
fn docs_ok() {
    struct Ns;
    impl zod_core::Namespace for Ns {
        const NAME: &'static str = "MyNs";
    }
    #[derive(ZodInputOnly)]
    #[zod(namespace = "Ns")]
    /// This is my rustdocs
    /// This is my second line
    struct Tuple(u8);

    assert_eq!(
        Tuple::export().unwrap().as_zod().to_string(),
        "\n// This is my rustdocs\n// This is my second line\nexport const Tuple = z.tuple([Rs.input.U8]);"
    );
}
