use pretty_assertions::assert_eq;
use zod::prelude::*;

#[test]
fn serde_rename_ok() {
    #[derive(Namespace)]
    struct Ns;

    #[derive(ZodInputOnly, serde::Deserialize)]
    #[zod(namespace = "Ns")]
    #[serde(rename = "YourStruct")]
    struct MyStruct {
        _value: u8,
    }
    assert_eq!(
        MyStruct::inline(),
        zod_core::Reference::<Kind::Input>::builder()
            .name("YourStruct")
            .ns("Ns")
            .build()
            .into()
    );

    const _: () = Ns::__ZOD_PRIVATE_INPUT___YourStruct;
}

#[test]
fn serde_rename_all_ok() {
    #[derive(Namespace)]
    struct Ns;

    #[derive(ZodInputOnly, serde::Deserialize)]
    #[zod(namespace = "Ns")]
    #[serde(rename_all = "UPPERCASE")]
    #[allow(dead_code)]
    struct MyStruct {
        value: u8,
    }
    assert_eq!(
        MyStruct::value().inner,
        z::ZodTypeInner::Object(
            z::ZodObject::<Kind::Input>::builder()
                .fields(vec![z::ZodNamedField::builder()
                    .name("VALUE")
                    .value(u8::inline())
                    .build()])
                .build()
        )
    )
}
