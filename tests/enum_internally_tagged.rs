use pretty_assertions::assert_eq;

mod test_utils;
use test_utils::*;

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

#[ignore]
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
        r#"{"A":{"s":""}}"#
    );

    compare_export::<Test>(
        "export const Test = z.lazy(() => z.union([z.object({ A: z.object({ s: Rs.String }) }), z.object({ B: z.object({ num: Rs.Usize }) })]));",
        "export type Test = {A: {s: Rs.String}} | {B: {num: Rs.Usize}};",
    );
}

#[ignore]
#[test]
fn enum_newtype() {
    test_case! {
        #[serde(tag = "type")]
        enum Test {
            A(String),
            B(Usize)
        }
    }

    assert_eq!(
        serde_json::to_string(&Test::A(String::new())).unwrap(),
        r#"{"A":""}"#
    );

    compare_export::<Test>(
    "export const Test = z.lazy(() => z.union([z.object({ A: Rs.String }), z.object({ B: Rs.Usize })]));",
    "export type Test = {A: Rs.String} | {B: Rs.Usize};",
    );
}
