use zod::prelude::*;

struct Ns;
impl zod_core::Namespace for Ns {
    const NAME: &'static str = "Custom_Ns";
}

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
