use pretty_assertions::assert_eq;

mod test_utils;
use test_utils::*;

#[test]
fn enum_adj_struct() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A { s: String },
            B { num: u16 },
        }
    }

    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(
        json,
        serde_json::json!({"type": "B", "content": { "num": 123 }})
    );

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [
            z.object({ type: z.literal("A"), content: z.object({s: Rs.String})}),
            z.object({type: z.literal("B"), content: z.object({ num: Rs.U16 })})
        ]));"#,
        r#"export type Test = { type: "A", content: {s: Rs.String} } | { type: "B", content: {num: Rs.U16} };"#,
    );
}

#[test]
fn enum_adj_struct_multiple_fields() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A { s: String, num: u16 },
            B { num: u16 },
        }
    }
    let json = serde_json::to_value(Test::A {
        s: String::from("abc"),
        num: 123,
    })
    .unwrap();
    assert_eq!(
        json,
        serde_json::json!({"type": "A", "content": {"s": "abc", "num": 123}})
    );

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [
        z.object({type: z.literal("A"), content: z.object({s: Rs.String, num: Rs.U16})}),
        z.object({type: z.literal("B"), content: z.object({num: Rs.U16})})
    ]));"#,
        r#"export type Test = { type: "A", content: {s: Rs.String, num: Rs.U16}} | {type: "B", content: {num: Rs.U16}};"#,
    );
}

#[test]
fn enum_adj_struct_multiple_fields_single() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A { s: String, num: u16 },
        }
    }
    let json = serde_json::to_value(Test::A {
        s: String::new(),
        num: 123,
    })
    .unwrap();
    assert_eq!(
        json,
        serde_json::json!({"type": "A", "content": {"s": "", "num": 123}})
    );

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [
        z.object({type: z.literal("A"), content: z.object({ s: Rs.String, num: Rs.U16 })})
    ]));"#,
        r#"export type Test = {type: "A", content: {s: Rs.String, num: Rs.U16}};"#,
    )
}

#[test]
fn enum_adj_tuple() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A(String),
            B(u16),
        }
    }
    let json = serde_json::to_value(Test::B(123)).unwrap();
    assert_eq!(json, serde_json::json!({"type": "B", "content": 123}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [
            z.object({type: z.literal("A"), content: Rs.String}),
            z.object({type: z.literal("B"), content: Rs.U16})
        ]));"#,
        r#"export type Test = {type: "A", content: Rs.String} | {type: "B", content: Rs.U16};"#,
    );
}

#[test]
fn enum_adj_tuple_multiple_fields() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A(u16, u16),
            B(String),
        }
    }

    let json = serde_json::to_value(Test::A(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!({"type": "A", "content": [123, 42]}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [
            z.object({type: z.literal("A"), content: Rs.Tuple2(Rs.U16, Rs.U16) }),
            z.object({type: z.literal("B"), content: Rs.String})
        ]));"#,
        r#"export type Test = {type: "A", content: Rs.Tuple2<Rs.U16, Rs.U16> } | {type: "B", content: Rs.String };"#,
    );
}

#[test]
fn enum_adj_tuple_multiple_fields_single_variant() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A(u16, u16),
        }
    }
    let json = serde_json::to_value(Test::A(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!({"type": "A", "content": [123, 42]}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [z.object({type: z.literal("A"), content: Rs.Tuple2(Rs.U16, Rs.U16)})]));"#,
        r#"export type Test = {type: "A", content: Rs.Tuple2<Rs.U16, Rs.U16> };"#,
    );
}
//
#[test]
fn enum_adj_tuple_single_variant() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A(u16),
        }
    }

    let json = serde_json::to_value(Test::A(123)).unwrap();
    assert_eq!(json, serde_json::json!({"type": "A", "content": 123}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [z.object({type: z.literal("A"), content: Rs.U16})]));"#,
        r#"export type Test = {type: "A", content: Rs.U16 };"#,
    );
}
#[test]
fn enum_adj_unit() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A,
            B,
        }
    }
    let json = serde_json::to_value(Test::B).unwrap();
    assert_eq!(json, serde_json::json!({"type": "B"}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [z.object({type: z.literal("A")}), z.object({type: z.literal("B")})]));"#,
        r#"export type Test = {type: "A" } | {type: "B" };"#,
    );
}
//
#[test]
fn enum_adj_unit_single_variant() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A,
        }
    }
    let json = serde_json::to_value(Test::A).unwrap();
    assert_eq!(json, serde_json::json!({ "type": "A"}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [z.object({type: z.literal("A")})]));"#,
        r#"export type Test = {type: "A" };"#,
    );
}

#[test]
fn enum_extern_struct() {
    test_case! {
        enum Test {
            A { s: String },
            B { num: u16 },
        }
    }

    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"B": {"num": 123}}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([z.object({A: z.object({s: Rs.String})}), z.object({B: z.object({num: Rs.U16})})]));"#,
        r#"export type Test = { A: {s: Rs.String}} | {B: {num: Rs.U16}};"#,
    );
}

#[test]
fn enum_extern_struct_multiple_fields() {
    test_case! {
        enum Test {
            A { s: String, num: u16 },
            B { num: u16 },
        }
    }
    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"B": {"num": 123}}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([z.object({A: z.object({s: Rs.String, num: Rs.U16})}), z.object({B: z.object({num: Rs.U16})})]));"#,
        r#"export type Test = { A: {s: Rs.String, num: Rs.U16}} | {B: {num: Rs.U16}};"#,
    );
}
#[test]
fn enum_extern_struct_multiple_fields_single_variant() {
    test_case! {
        enum Test {
            A { s: String, num: u16 },
        }
    }
    let json = serde_json::to_value(Test::A {
        s: String::new(),
        num: 123,
    })
    .unwrap();
    assert_eq!(json, serde_json::json!({"A": {"s": "", "num": 123}}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([z.object({A: z.object({s: Rs.String, num: Rs.U16})})]));"#,
        r#"export type Test = { A: {s: Rs.String, num: Rs.U16}};"#,
    );
}

#[test]
fn enum_extern_tuple() {
    test_case! {
        enum Test {
            A(String),
            B(u16),
        }
    }
    let json = serde_json::to_value(Test::B(123)).unwrap();
    assert_eq!(json, serde_json::json!({"B": 123}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([z.object({A: Rs.String}), z.object({B: Rs.U16})]));"#,
        r#"export type Test = { A: Rs.String} | {B: Rs.U16};"#,
    );
}

#[test]
fn enum_extern_tuple_multiple_fields() {
    test_case! {
        enum Test {
            A(u16, u16),
            B(String),
        }
    }
    let json = serde_json::to_value(Test::A(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!({"A": [123, 42]}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([z.object({A: z.tuple([Rs.U16, Rs.U16])}), z.object({B: Rs.String})]));"#,
        r#"export type Test = { A: [Rs.U16, Rs.U16]} | {B: Rs.String};"#,
    );
}

#[test]
fn enum_extern_tuple_multiple_fields_single_variant() {
    test_case! {
        enum Test {
            A(u16, u16),
        }
    }
    let json = serde_json::to_value(Test::A(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!({"A": [123, 42]}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([z.object({A: z.tuple([Rs.U16, Rs.U16])})]));"#,
        r#"export type Test = { A: [Rs.U16, Rs.U16]};"#,
    );
}

#[test]
fn enum_extern_tuple_single_single_field_single_variant() {
    test_case! {
        enum Test {
            A(u16),
        }
    }
    let json = serde_json::to_value(Test::A(123)).unwrap();
    assert_eq!(json, serde_json::json!({"A": 123}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([z.object({A: Rs.U16})]));"#,
        r#"export type Test = { A: Rs.U16 };"#,
    );
}

#[test]
fn enum_extern_unit() {
    test_case! {
        enum Test {
            A,
            B,
        }
    }

    let json = serde_json::to_value(Test::B).unwrap();
    assert_eq!(json, serde_json::json!("B"));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([z.literal("A"), z.literal("B")]));"#,
        r#"export type Test = "A" | "B";"#,
    );
}

#[test]
fn enum_extern_unit_single_variant() {
    test_case! {
        enum Test {
            A,
        }
    }
    let json = serde_json::to_value(Test::A).unwrap();
    assert_eq!(json, serde_json::json!("A"));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([z.literal("A")]));"#,
        r#"export type Test = "A";"#,
    );
}

#[test]
fn enum_intern_struct() {
    test_case! {
        #[serde(tag = "type")]
        enum Test {
            A { s: String },
            B { num: u16 },
        }
    }
    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"type": "B", "num": 123}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [
            z.object({ type: z.literal("A"), s: Rs.String }),
            z.object({ type: z.literal("B"), num: Rs.U16 })
        ]));"#,
        r#"export type Test = {type: "A", s: Rs.String} | {type: "B", num: Rs.U16};"#,
    );
}

#[test]
fn enum_intern_struct_multiple_fields() {
    test_case! {
        #[serde(tag = "type")]
        enum Test {
            A { s: String, num: u16 },
            B { num: u16 },
        }
    }

    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"type": "B", "num": 123}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [
            z.object({ type: z.literal("A"), s: Rs.String, num: Rs.U16 }),
            z.object({ type: z.literal("B"), num: Rs.U16})
        ]));"#,
        r#"export type Test = { type: "A", s: Rs.String, num: Rs.U16 } | { type: "B", num: Rs.U16 };"#,
    );
}

#[test]
fn enum_intern_struct_multiple_fields_single_variant() {
    test_case! {
        #[serde(tag = "type")]
        enum Test {
            A { s: String, num: u16 },
        }
    }

    let json = serde_json::to_value(Test::A {
        s: String::new(),
        num: 123,
    })
    .unwrap();
    assert_eq!(json, serde_json::json!({"type": "A", "s": "", "num": 123}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [
            z.object({ type: z.literal("A"), s: Rs.String, num: Rs.U16 })
        ]));"#,
        r#"export type Test = { type: "A", s: Rs.String, num: Rs.U16 };"#,
    );
}

#[test]
fn enum_intern_unit() {
    test_case! {
        #[serde(tag = "type")]
        enum Test {
            A,
            B,
        }
    }
    let json = serde_json::to_value(Test::B).unwrap();
    assert_eq!(json, serde_json::json!({"type": "B"}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [
            z.object({ type: z.literal("A") }),
            z.object({ type: z.literal("B") })
        ]));"#,
        r#"export type Test = { type: "A" } | { type: "B" };"#,
    );
}

#[test]
fn enum_intern_unit_single_variant() {
    test_case! {
        #[serde(tag = "type")]
        enum Test {
            A,
        }
    }

    let json = serde_json::to_value(Test::A).unwrap();
    assert_eq!(json, serde_json::json!({ "type": "A"}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.discriminatedUnion("type", [
            z.object({ type: z.literal("A") })
        ]));"#,
        r#"export type Test = { type: "A" };"#,
    );
}

#[test]
fn enum_untagged_struct() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A { s: String },
            B { num: u16 },
        }
    }
    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"num": 123}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([
            z.object({ s: Rs.String }),
            z.object({ num: Rs.U16})
        ]));"#,
        r#"export type Test = { s: Rs.String } | { num: Rs.U16 };"#,
    );
}
//
#[test]
fn enum_untagged_struct_multiple_fields() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A { s: String, num: u16 },
            B { num: u16 },
        }
    }
    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"num": 123}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([
            z.object({ s: Rs.String, num: Rs.U16 }),
            z.object({ num: Rs.U16})
        ]));"#,
        r#"export type Test = { s: Rs.String, num: Rs.U16 } | { num: Rs.U16 };"#,
    );
}

#[test]
fn enum_untagged_struct_multiple_fields_single_variant() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A { s: String, num: u16 },
        }
    }
    let json = serde_json::to_value(Test::A {
        s: String::new(),
        num: 123,
    })
    .unwrap();
    assert_eq!(json, serde_json::json!({"s": "", "num": 123}));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([
            z.object({ s: Rs.String, num: Rs.U16 })
        ]));"#,
        r#"export type Test = { s: Rs.String, num: Rs.U16 };"#,
    );
}

#[test]
fn enum_untagged_tuple() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A(String),
            B(u16),
        }
    }
    let json = serde_json::to_value(Test::B(123)).unwrap();
    assert_eq!(json, serde_json::json!(123));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([
            Rs.String,
            Rs.U16
        ]));"#,
        r#"export type Test = Rs.String | Rs.U16;"#,
    );
}

#[test]
fn enum_untagged_tuple_multiple_fields() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A(u16, u16),
            B(String),
        }
    }
    let json = serde_json::to_value(Test::A(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!([123, 42]));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([
            z.tuple([Rs.U16, Rs.U16]),
            Rs.String
        ]));"#,
        r#"export type Test = [Rs.U16, Rs.U16] | Rs.String;"#,
    );
}
#[test]
fn enum_untagged_tuple_multiple_fields_single_variant() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A(u16, u16),
        }
    }

    let json = serde_json::to_value(Test::A(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!([123, 42]));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([
            z.tuple([Rs.U16, Rs.U16])
        ]));"#,
        r#"export type Test = [Rs.U16, Rs.U16];"#,
    );
}

#[test]
fn enum_untagged_tuple_single_field_single_variant() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A(u16),
        }
    }
    let json = serde_json::to_value(Test::A(123)).unwrap();
    assert_eq!(json, serde_json::json!(123));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([
            Rs.U16
        ]));"#,
        r#"export type Test = Rs.U16;"#,
    );
}

#[test]
fn enum_untagged_unit() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A,
            B,
        }
    }
    let json = serde_json::to_value(Test::B).unwrap();
    assert_eq!(json, serde_json::json!(null));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([
            Rs.Unit, Rs.Unit
        ]));"#,
        r#"export type Test = Rs.Unit|Rs.Unit;"#,
    );
}

#[test]
fn enum_untagged_unit_single_variant() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A,
        }
    }
    let json = serde_json::to_value(Test::A).unwrap();
    assert_eq!(json, serde_json::json!(null));

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([
            Rs.Unit
        ]));"#,
        r#"export type Test = Rs.Unit;"#,
    );
}
