use pretty_assertions::assert_eq;

mod test_utils;
use test_utils::*;
use zod::ResponseType;

#[test]
fn enum_unit() {
    test_case! {
        #[serde(tag = "type")]
        enum Test {
            A,
            B,
        }
    }

    assert_eq!(serde_json::to_string(&Test::A).unwrap(), r#"{"type":"A"}"#);

    compare_export::<Test>(
        "export const Test = z.lazy(() => z.discriminatedUnion(\"type\", [z.object({ type: z.literal(\"A\") }), z.object({ type: z.literal(\"B\") })]));",
        "export type Test = { type: \"A\" } | { type: \"B\" };",
    );
}

#[test]
fn enum_struct() {
    test_case! {
        #[serde(tag = "type")]
        enum Test {
            A { s: String },
            B { num: Usize },
        }
    }

    assert_eq!(
        serde_json::to_string(&Test::A { s: String::new() }).unwrap(),
        r#"{"type":"A","s":""}"#
    );

    compare_export::<Test>(
        "export const Test = z.lazy(() => z.discriminatedUnion(\"type\", [z.object({ type: z.literal(\"A\"), s: Rs.String }), z.object({ type: z.literal(\"B\"), num: Rs.Usize }) ]));",
        "export type Test = { type: \"A\", s: Rs.String} | { type: \"B\", num: Rs.Usize };",
    );
}

#[test]
fn enum_newtype() {
    test_case! {
        #[derive(serde::Deserialize, ResponseType)]
        struct MyStruct {
            name: String,
            age: u8
        }

        #[derive(serde::Serialize, serde::Deserialize, RequestType, ResponseType)]
        #[zod(namespace = "Ns")]
        #[serde(tag = "type")]
        enum Test {
            A(MyStruct),
            BackUp([&'static str;5]),
        }

    }

    assert_eq!(
        serde_json::to_string(&Test::A(MyStruct {
            name: String::from("bob"),
            age: 42
        }))
        .unwrap(),
        r#"{"type":"A","name":"bob","age":42}"#
    );

    compare_export::<Test>(
    "export const Test = z.lazy(() => z.union([z.object({ A: Rs.String }), z.object({ B: Rs.Usize })]));",
    "export type Test = {A: Rs.String} | {B: Rs.Usize};",
    );
}
