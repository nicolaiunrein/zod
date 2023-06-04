#![allow(dead_code)]
use pretty_assertions::assert_eq;
use zod::prelude::*;

struct Ns;
impl zod_core::Namespace for Ns {
    const NAME: &'static str = "Custom_Ns";
}

#[derive(ZodInputOnly)]
#[zod(namespace = "Ns")]
struct Other {
    value: String,
}

#[derive(ZodInputOnly)]
#[zod(namespace = "Ns")]
struct StructNamed {
    #[zod(override_input_with = "abc")]
    value1: bool,

    #[zod(override_input_with = "Other::value")]
    value2: bool,

    #[zod(override_with = "Other::value")]
    value3: bool,
}

#[derive(ZodInputOnly)]
#[zod(namespace = "Ns")]
struct TupleStruct(
    #[zod(override_input_with = "abc")] bool,
    #[zod(override_input_with = "Other::value")] bool,
    #[zod(override_with = "Other::value")] bool,
);

// TODO: Enum

fn abc() -> z::ZodNumber {
    z::ZodNumber
}

#[test]
fn override_fields_on_named_struct() {
    assert_eq!(
        StructNamed::export().unwrap().as_zod().to_string(),
        format!(
            "export const StructNamed = z.object({{ value1: z.number(), value2: {}, value3: {} }});",
            Other::value().as_zod(),
            Other::value().as_zod()
        )
    )
}

#[test]
fn override_fields_on_tuple_struct() {
    assert_eq!(
        TupleStruct::export().unwrap().as_zod().to_string(),
        format!(
            "export const TupleStruct = z.tuple([z.number(), {}, {}]);",
            Other::value().as_zod(),
            Other::value().as_zod()
        )
    )
}

// TODO: make a test case to confirm that override values are present in collected exports
