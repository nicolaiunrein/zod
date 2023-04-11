use pretty_assertions::assert_eq;

mod test_utils;
use test_utils::*;
use zod::ResponseType;

#[test]
fn enum_unit() {
    test_case! {
        #[serde(tag = "type", content = "value")]
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
        #[serde(tag = "type", content = "content")]
        enum Test {
            A { s: String },
            B { num: Usize },
        }
    }

    assert_eq!(
        serde_json::to_string(&Test::A { s: String::new() }).unwrap(),
        r#"{"type":"A","content":{"s":""}}"#
    );

    compare_export::<Test>(
        "export const Test = z.lazy(() => z.discriminatedUnion(\"type\", [z.object({ type: z.literal(\"A\"), content: z.object({ s: Rs.String }) }), z.object({ type: z.literal(\"B\"), content: z.object({ num: Rs.Usize }) }) ]));",
        "export type Test = { type: \"A\", content: { s: Rs.String } } | { type: \"B\", content: { num: Rs.Usize } };",
    );
}

#[test]
fn enum_newtype() {
    test_case! {
        #[derive(serde::Deserialize, ResponseType)]
        struct Both {
            name: String,
            age: u8
        }
        #[derive(serde::Serialize, serde::Deserialize, RequestType, ResponseType)]
        #[zod(namespace = "Ns")]
        struct AgeOnly{
            age: u8
        }


        #[derive(serde::Serialize, serde::Deserialize, RequestType, ResponseType )]
        #[zod(namespace = "Ns")]
        #[serde(tag = "type", content = "content")]
        enum Test {
            A(Both),
            B(AgeOnly),
        }

    }

    assert_eq!(
        serde_json::to_string(&Test::A(Both {
            name: String::from("bob"),
            age: 42
        }))
        .unwrap(),
        r#"{"type":"A","content":{"name":"bob","age":42}}"#
    );

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [z.object({ type: z.literal("A"), content: Ns.Both }), z.object({ type: z.literal("B"), content: Ns.AgeOnly })]));"#,
        r#"export type Test = { type: "A", content: Ns.Both } | { type: "B", content: Ns.AgeOnly };"#,
    );
}

#[test]
fn enum_tuple() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A(Usize, String),
            B(String, Usize),
        }

    }

    assert_eq!(
        serde_json::to_string(&Test::A(Usize(1), String::from("abc"))).unwrap(),
        r#"{"type":"A","content":["1","abc"]}"#
    );

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [z.object({ type: z.literal("A"), content: z.tuple([Rs.Usize, Rs.String]) }), z.object({ type: z.literal("B"), content: z.tuple([Rs.String, Rs.Usize])})]));"#,
        r#"export type Test = { type: "A", content: [Rs.Usize, Rs.String]} | { type: "B", content: [Rs.String, Rs.Usize]};"#,
    );
}
