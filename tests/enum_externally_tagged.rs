use pretty_assertions::assert_eq;

mod test_utils;
use test_utils::*;

#[test]
fn enum_struct() {
    test_case! {
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

#[test]
fn enum_newtype() {
    test_case! {
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

#[test]
fn enum_tuple() {
    test_case! {
        enum Test {
            A(String, Usize),
            B(Usize, String)
        }
    }

    assert_eq!(
        serde_json::to_string(&Test::A(String::new(), Usize(42))).unwrap(),
        r#"{"A":["","42"]}"#
    );

    compare_export::<Test>(
    "export const Test = z.lazy(() => z.union([z.object({ A: z.tuple([ Rs.String, Rs.Usize]) }), z.object({ B: z.tuple([ Rs.Usize, Rs.String]) })]));",
    "export type Test = { A: [ Rs.String, Rs.Usize ]} | { B: [ Rs.Usize, Rs.String ]};",
    );
}

#[test]
fn enum_unit() {
    test_case! {
        enum Test {
            A,
            B,
        }
    }

    assert_eq!(serde_json::to_string(&Test::A).unwrap(), r#""A""#);

    compare_export::<Test>(
        "export const Test = z.lazy(() => z.union([z.literal(\"A\"), z.literal(\"B\")]));",
        "export type Test = \"A\" | \"B\";",
    );
}
