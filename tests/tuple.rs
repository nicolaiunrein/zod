use pretty_assertions::assert_eq;

use zod::prelude::*;

#[test]
fn tuple_empty_ok() {
    struct Ns;
    impl zod_core::Namespace for Ns {
        const NAME: &'static str = "MyNs";
    }
    #[derive(Zod)]
    #[zod(namespace = "Ns")]
    struct Tuple();

    assert_eq!(
        <Tuple as TypeExt<Kind::Input>>::export()
            .unwrap()
            .as_zod()
            .to_string(),
        "export const Tuple = z.tuple([]);"
    );

    assert_eq!(
        <Tuple as TypeExt<Kind::Input>>::inline()
            .as_zod()
            .to_string(),
        "MyNs.input.Tuple"
    );
}

#[test]
fn tuple1_ok() {
    struct Ns;
    impl zod_core::Namespace for Ns {
        const NAME: &'static str = "MyNs";
    }
    #[derive(Zod)]
    #[zod(namespace = "Ns")]
    struct Tuple(u8);

    assert_eq!(
        <Tuple as TypeExt<Kind::Input>>::export()
            .unwrap()
            .as_zod()
            .to_string(),
        "export const Tuple = z.tuple([Rs.input.U8]);"
    );

    assert_eq!(
        <Tuple as TypeExt<Kind::Input>>::inline()
            .as_zod()
            .to_string(),
        "MyNs.input.Tuple"
    );
}

#[test]
fn tuple2_ok() {
    struct Ns;
    impl zod_core::Namespace for Ns {
        const NAME: &'static str = "MyNs";
    }
    #[derive(Zod)]
    #[zod(namespace = "Ns")]
    struct Tuple(u8, String);

    assert_eq!(
        <Tuple as TypeExt<Kind::Input>>::export()
            .unwrap()
            .as_zod()
            .to_string(),
        "export const Tuple = z.tuple([Rs.input.U8, Rs.input.String]);"
    );

    assert_eq!(
        <Tuple as TypeExt<Kind::Input>>::inline()
            .as_zod()
            .to_string(),
        "MyNs.input.Tuple"
    );
}
