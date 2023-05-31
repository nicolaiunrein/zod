#![allow(dead_code)]
use pretty_assertions::assert_eq;
use zod_core::z;
use zod_core::Kind;
use zod_core::TypeExt;
use zod_derive_experiments::Zod;

struct Ns;
impl zod_core::Namespace for Ns {
    const NAME: &'static str = "Custom_Ns";
}

#[derive(Zod)]
#[zod(namespace = "Ns")]
enum ExternallyTagged {
    Unit,
    Tuple0(),
    Tuple1(String),
    Tuple2(String, u8),
    Struct0 {},
    Struct1 { inner: String },
    Struct2 { inner_string: String, inner_u8: u8 },
}

#[derive(Zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[serde(untagged)]
enum Untagged {
    Unit,
    Tuple0(),
    Tuple1(String),
    Tuple2(String, u8),
    Struct0 {},
    Struct1 { inner: String },
    Struct2 { inner_string: String, inner_u8: u8 },
}

#[derive(Zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[serde(tag = "my_tag")]
enum InternallyTagged {
    Unit,
    Struct0 {},
    Struct1 { inner: String },
    Struct2 { inner_string: String, inner_u8: u8 },
}

#[derive(Zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[serde(tag = "my_tag", content = "my_content")]
enum AdjacentlyTagged {
    Unit,
    Tuple0(),
    Tuple1(String),
    Tuple2(String, u8),
    Struct0 {},
    Struct1 { inner: String },
    Struct2 { inner_string: String, inner_u8: u8 },
}

#[test]
fn externally_tagged_ok() {
    let export: z::ZodExport<Kind::Input> = ExternallyTagged::export().unwrap();
    let variants = [
        "z.literal(\"Unit\")",
        "z.object({ Tuple0: z.tuple([]) })",
        "z.object({ Tuple1: z.tuple([Rs.input.String]) })",
        "z.object({ Tuple2: z.tuple([Rs.input.String, Rs.input.U8]) })",
        "z.object({ Struct0: z.object({}) })",
        "z.object({ Struct1: z.object({ inner: Rs.input.String }) })",
        "z.object({ Struct2: z.object({ inner_string: Rs.input.String, inner_u8: Rs.input.U8 }) })",
    ];

    assert_eq!(
        zod_core::z::Zod(&export).to_string(),
        format!(
            "export const ExternallyTagged = z.union([{}]);",
            variants.join(", ")
        )
    );
}

#[test]
fn untagged_tagged_ok() {
    let export: z::ZodExport<Kind::Input> = Untagged::export().unwrap();
    let variants = [
        "z.literal(\"Unit\")",
        "z.tuple([])",
        "z.tuple([Rs.input.String])",
        "z.tuple([Rs.input.String, Rs.input.U8])",
        "z.object({})",
        "z.object({ inner: Rs.input.String })",
        "z.object({ inner_string: Rs.input.String, inner_u8: Rs.input.U8 })",
    ];

    assert_eq!(
        zod_core::z::Zod(&export).to_string(),
        format!(
            "export const Untagged = z.union([{}]);",
            variants.join(", ")
        )
    );
}

#[test]
fn internally_tagged_ok() {
    let export: z::ZodExport<Kind::Input> = InternallyTagged::export().unwrap();
    let variants = [
        "z.object({ my_tag: z.literal(\"Unit\") })",
        "z.object({ my_tag: z.literal(\"Struct0\") })",
        "z.object({ my_tag: z.literal(\"Struct1\"), inner: Rs.input.String })",
        "z.object({ my_tag: z.literal(\"Struct2\"), inner_string: Rs.input.String, inner_u8: Rs.input.U8 })",
    ];

    assert_eq!(
        zod_core::z::Zod(&export).to_string(),
        format!(
            "export const InternallyTagged = z.discriminatedUnion(\"my_tag\", [{}]);",
            variants.join(", ")
        )
    );
}

#[test]
fn adjacently_tagged_ok() {
    let export: z::ZodExport<Kind::Input> = AdjacentlyTagged::export().unwrap();
    let variants = [
        "z.object({ my_tag: z.literal(\"Unit\") })",
        "z.object({ my_tag: z.literal(\"Tuple0\"), my_content: z.tuple([]) })",
        "z.object({ my_tag: z.literal(\"Tuple1\"), my_content: z.tuple([Rs.input.String]) })",
        "z.object({ my_tag: z.literal(\"Tuple2\"), my_content: z.tuple([Rs.input.String, Rs.input.U8]) })",
        "z.object({ my_tag: z.literal(\"Struct0\"), my_content: z.object({}) })",
        "z.object({ my_tag: z.literal(\"Struct1\"), my_content: z.object({ inner: Rs.input.String }) })",
        "z.object({ my_tag: z.literal(\"Struct2\"), my_content: z.object({ inner_string: Rs.input.String, inner_u8: Rs.input.U8 }) })",
    ];

    assert_eq!(
        zod_core::z::Zod(&export).to_string(),
        format!(
            "export const AdjacentlyTagged = z.discriminatedUnion(\"my_tag\", [{}]);",
            variants.join(", ")
        )
    );
}
