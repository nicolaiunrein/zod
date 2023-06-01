use zod_derive_experiments::Namespace;
use zod_derive_experiments::Zod;
use zod_derive_experiments::ZodInputOnly;
use zod_derive_experiments::ZodOutputOnly;

#[derive(Namespace)]
#[zod(name = "Custom_Ns")]
struct Ns;

#[derive(Zod)]
#[zod(namespace = "Ns")]
struct StructIo {
    _value: u8,
}

#[derive(ZodInputOnly)]
#[zod(namespace = "Ns")]
struct StructInputOnly {
    pub _value: u8,
}
//
#[derive(ZodOutputOnly)]
#[zod(namespace = "Ns")]
struct StructOutputOnly {
    pub _value: u8,
}

mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use zod_core::prelude::*;

    #[test]
    fn gives_correct_reference() {
        assert_eq!(
            StructInputOnly::inline(),
            zod_core::Reference::<Kind::Input>::builder()
                .name("StructInputOnly")
                .ns("Custom_Ns")
                .build()
                .into()
        );
    }
}
