use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

macro_rules! test_case {
    ($($decl: tt)+) => {
        #[derive(zod, serde::Serialize)]
        #[zod(namespace = "Ns")]
        #[allow(dead_code)]
        $($decl)+

        struct Ns {}

        impl Namespace for Ns {
            const NAME: &'static str = "Ns";
            type Req = NsReq;
        }

        #[derive(serde::Deserialize)]
        struct NsReq;
    };
}

#[test]
fn enum_adj_struct() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A { s: String },
            B { num: usize },
        }
    }

    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(
        json,
        serde_json::json!({"type": "B", "content": { "num": 123 }})
    );

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.discriminatedUnion(\"type\", [z.object({{ type: z.literal(\"A\"), content: z.object({{ s: {string_schema} }}) }}), z.object({{ type: z.literal(\"B\"), content: z.object({{ num: {number_schema} }}) }})])")
    );
    assert_eq!(
        Test::type_def(),
        "{ type: \"A\", content: { s: string } } | { type: \"B\", content: { num: number } }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_adj_struct_multiple_fields() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A { s: String, num: usize },
            B { num: usize },
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

    let string_schema = String::schema();
    let number_schema = usize::schema();

    assert_eq!(
        Test::schema(),
        format!("z.discriminatedUnion(\"type\", [z.object({{ type: z.literal(\"A\"), content: z.object({{ s: {string_schema}, num: {number_schema} }}) }}), z.object({{ type: z.literal(\"B\"), content: z.object({{ num: {number_schema} }}) }})])")
    );
    assert_eq!(
        Test::type_def(),
        "{ type: \"A\", content: { s: string, num: number } } | { type: \"B\", content: { num: number } }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_adj_struct_multiple_fields_single() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A { s: String, num: usize },
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

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.object({{ type: z.literal(\"A\"), content: z.object({{ s: {string_schema}, num: {number_schema} }}) }})")
    );
    assert_eq!(
        Test::type_def(),
        "{ type: \"A\", content: { s: string, num: number } }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}
