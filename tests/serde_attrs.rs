#![allow(dead_code)]
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
        value: u8,
    }
    assert_eq!(
        MyStruct::inline().as_zod().to_string(),
        "Ns.input.YourStruct"
    );
    assert_eq!(
        MyStruct::export().unwrap().as_zod().to_string(),
        "export const YourStruct = z.object({ value: Rs.input.U8 });"
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
    struct MyStruct {
        some_value: u8,
    }
    assert_eq!(MyStruct::inline().as_zod().to_string(), "Ns.input.MyStruct");
    assert_eq!(
        MyStruct::export().unwrap().as_zod().to_string(),
        "export const MyStruct = z.object({ SOME_VALUE: Rs.input.U8 });"
    );
}

#[test]
fn default_named_fields_ok() {
    #[derive(Namespace)]
    struct Ns;

    fn default_value() -> u8 {
        42
    }

    #[derive(ZodInputOnly, serde::Deserialize)]
    #[zod(namespace = "Ns")]
    struct MyStruct {
        #[serde(default)]
        value: u8,

        #[serde(default = "default_value")]
        value42: u8,
    }

    assert_eq!(
        MyStruct::export().unwrap().as_zod().to_string(),
        "export const MyStruct = z.object({ value: Rs.input.U8.optional(), value42: Rs.input.U8.optional() });"
    );
}

#[test]
fn default_tuple_fields_ok() {
    #[derive(Namespace)]
    struct Ns;

    fn default_value() -> u8 {
        42
    }

    #[derive(ZodInputOnly, serde::Deserialize)]
    #[zod(namespace = "Ns")]
    struct MyStruct(#[serde(default)] u8, #[serde(default = "default_value")] u8);

    assert_eq!(
        MyStruct::export().unwrap().as_zod().to_string(),
        "export const MyStruct = z.tuple([Rs.input.U8.optional(), Rs.input.U8.optional()]);"
    );
}

#[test]
fn skip_struct_fields() {
    #[derive(Namespace)]
    struct Ns;

    #[derive(Zod, serde::Deserialize)]
    #[zod(namespace = "Ns")]
    struct MyStruct {
        value: u8,
        #[serde(skip)]
        skipped: bool,

        #[serde(skip_serializing_if = "bool::is_false")]
        skipped_if: bool,
    }

    #[derive(ZodInputOnly, serde::Deserialize)]
    #[zod(namespace = "Ns")]
    struct MyTuple(u8, #[serde(skip)] bool);

    assert_eq!(
        <MyStruct as TypeExt<Kind::Input>>::export()
            .unwrap()
            .as_zod()
            .to_string(),
        "export const MyStruct = z.object({ value: Rs.input.U8, skipped_if: Rs.input.Bool });"
    );

    assert_eq!(
        <MyStruct as TypeExt<Kind::Output>>::export()
            .unwrap()
            .as_zod()
            .to_string(),
        "export const MyStruct = z.object({ value: Rs.output.U8, skipped_if: Rs.output.Bool.optional() });"
    );

    assert_eq!(
        MyTuple::export().unwrap().as_zod().to_string(),
        "export const MyTuple = z.tuple([Rs.input.U8]);"
    );
}

#[test]
fn skip_enum_fields() {
    #[derive(Namespace)]
    struct Ns;

    #[derive(ZodInputOnly, serde::Deserialize)]
    #[zod(namespace = "Ns")]
    enum MyEnum {
        Tuple0(#[serde(skip)] bool),
        Tuple1(u8, #[serde(skip)] bool),
        Named {
            value: u8,
            #[serde(skip)]
            skipped: bool,
        },
    }

    assert_eq!(
        MyEnum::export().unwrap().as_zod().to_string(),
        "export const MyEnum = z.union([z.object({ Tuple0: z.tuple([]) }), z.object({ Tuple1: z.tuple([Rs.input.U8]) }), z.object({ Named: z.object({ value: Rs.input.U8 }) })]);"
    );
}
