use zod_derive_experiments::Zod;
use zod_derive_experiments::ZodInputOnly;
use zod_derive_experiments::ZodOutputOnly;

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
    use zod_core::Kind;
    use zod_core::Reference;
    use zod_core::Type;

    #[test]
    fn gives_correct_reference() {
        assert_eq!(
            StructInputOnly::get_ref(),
            Reference::<Kind::Input>::builder()
                .name("StructInputOnly")
                .ns("Custom_Ns")
                .build()
                .into()
        );
    }
}
