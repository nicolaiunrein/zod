use pretty_assertions::assert_eq;
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

#[test]
fn struct_input_ok() {
    assert_eq!(
        StructInputOnly::inline().as_zod().to_string(),
        "Custom_Ns.input.StructInputOnly"
    );
}

#[test]
fn struct_output_ok() {
    assert_eq!(
        StructOutputOnly::inline().as_zod().to_string(),
        "Custom_Ns.output.StructOutputOnly"
    );
}
