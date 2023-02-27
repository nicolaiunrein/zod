#![cfg(test)]
use pretty_assertions::assert_eq;
use remotely::test_case;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

fn discriminated_union(t: impl AsRef<str>, items: &[impl AsRef<str>]) -> String {
    format!(
        "z.discriminatedUnion(\"{}\", [{}])",
        t.as_ref(),
        items
            .iter()
            .map(|i| i.as_ref())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn object(fields: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    let inner = fields
        .iter()
        .map(|(k, v)| format!("{}: {}", k.as_ref(), v.as_ref()))
        .collect::<Vec<_>>();
    format!("z.object({{ {} }})", inner.join(", "))
}

fn literal(inner: impl AsRef<str>) -> String {
    format!("z.literal({})", inner.as_ref())
}

fn escape(inner: impl AsRef<str>) -> String {
    format!("\"{}\"", inner.as_ref())
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
    let expected =
        discriminated_union("type", &[
                            object(&[
                                   ("type", literal(escape("A"))),
                                   ("content", object(&[
                                                      ("s", string_schema)
                                   ]))
                            ]),
            // format!("z.object({{ type: z.literal(\"A\"), content: z.object({{ s: {string_schema} }}) }})"),
            format!("z.object({{ type: z.literal(\"B\"), content: z.object({{ num: {number_schema} }}) }})")
        ]);

    assert_eq!(Test::schema(), expected);
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

#[test]
fn enum_adj_tuple() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A(String),
            B(usize),
        }
    }
    let json = serde_json::to_value(Test::B(123)).unwrap();
    assert_eq!(json, serde_json::json!({"type": "B", "content": 123}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.discriminatedUnion(\"type\", [z.object({{ type: z.literal(\"A\"), content: {string_schema}}}), z.object({{ type: z.literal(\"B\"), content: {number_schema}}})])")
    );
    assert_eq!(
        Test::type_def(),
        "{ type: \"A\", content: string } | { type: \"B\", content: number }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_adj_tuple_multiple_fields() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A(usize, usize),
            B(String),
        }
    }

    let json = serde_json::to_value(Test::A(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!({"type": "A", "content": [123, 42]}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.discriminatedUnion(\"type\", [z.object({{ type: z.literal(\"A\"), content: z.tuple([{number_schema}, {number_schema}])}}), z.object({{ type: z.literal(\"B\"), content: {string_schema}}})])")
    );
    assert_eq!(
        Test::type_def(),
        "{ type: \"A\", content: [number, number] } | { type: \"B\", content: string }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_adj_tuple_multiple_fields_single_variant() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A(usize, usize),
        }
    }
    let json = serde_json::to_value(Test::A(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!({"type": "A", "content": [123, 42]}));

    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.object({{ type: z.literal(\"A\"), content: z.tuple([{number_schema}, {number_schema}])}})")
    );
    assert_eq!(
        Test::type_def(),
        "{ type: \"A\", content: [number, number] }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_adj_tuple_single_variant() {
    test_case! {
        #[serde(tag = "type", content = "content")]
        enum Test {
            A(usize),
        }
    }

    let json = serde_json::to_value(Test::A(123)).unwrap();
    assert_eq!(json, serde_json::json!({"type": "A", "content": 123}));

    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.object({{ type: z.literal(\"A\"), content: {number_schema}}})")
    );
    assert_eq!(Test::type_def(), "{ type: \"A\", content: number }");
    assert_eq!(Test::type_name(), "Ns.Test");
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

    assert_eq!(
        Test::schema(),
        format!(
           "z.discriminatedUnion(\"type\", [z.object({{ type: z.literal(\"A\") }}), z.object({{ type: z.literal(\"B\") }})])"
        )
    );
    assert_eq!(Test::type_def(), "{ type: \"A\" } | { type: \"B\" }");
    assert_eq!(Test::type_name(), "Ns.Test");
}

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

    assert_eq!(
        Test::schema(),
        format!("z.object({{ type: z.literal(\"A\") }})")
    );
    assert_eq!(Test::type_def(), "{ type: \"A\" }");
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_extern_struct() {
    test_case! {
        enum Test {
            A { s: String },
            B { num: usize },
        }
    }

    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"B": {"num": 123}}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.union([z.object({{A: z.object({{ s: {string_schema} }}) }}), z.object({{B: z.object({{ num: {number_schema} }}) }})])")
    );
    assert_eq!(
        Test::type_def(),
        "{ A: { s: string } } | { B: { num: number } }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_extern_struct_multiple_fields() {
    test_case! {
        enum Test {
            A { s: String, num: usize },
            B { num: usize },
        }
    }
    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"B": {"num": 123}}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.union([z.object({{A: z.object({{ s: {string_schema}, num: {number_schema} }}) }}), z.object({{B: z.object({{ num: {number_schema} }}) }})])")
    );
    assert_eq!(
        Test::type_def(),
        "{ A: { s: string, num: number } } | { B: { num: number } }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_extern_struct_multiple_fields_single_variant() {
    test_case! {
        enum Test {
            A { s: String, num: usize },
        }
    }
    let json = serde_json::to_value(Test::A {
        s: String::new(),
        num: 123,
    })
    .unwrap();
    assert_eq!(json, serde_json::json!({"A": {"s": "", "num": 123}}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.object({{A: z.object({{ s: {string_schema}, num: {number_schema} }}) }})")
    );
    assert_eq!(Test::type_def(), "{ A: { s: string, num: number } }");
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_extern_tuple() {
    test_case! {
        enum Test {
            A(String),
            B(usize),
        }
    }
    let json = serde_json::to_value(Test::B(123)).unwrap();
    assert_eq!(json, serde_json::json!({"B": 123}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.union([z.object({{A: {string_schema}}}), z.object({{B: {number_schema}}})])")
    );
    assert_eq!(Test::type_def(), "{ A: string } | { B: number }");
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_extern_tuple_multiple_fields() {
    test_case! {
        enum Test {
            A(usize, usize),
            B(String),
        }
    }
    let json = serde_json::to_value(Test::A(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!({"A": [123, 42]}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.union([z.object({{A: z.tuple([{number_schema}, {number_schema}])}}), z.object({{B: {string_schema}}})])")
    );
    assert_eq!(Test::type_def(), "{ A: [number, number] } | { B: string }");
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_extern_tuple_multiple_fields_single_variant() {
    test_case! {
        enum Test {
            A(usize, usize),
        }
    }
    let json = serde_json::to_value(Test::A(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!({"A": [123, 42]}));

    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.object({{A: z.tuple([{number_schema}, {number_schema}])}})")
    );
    assert_eq!(Test::type_def(), "{ A: [number, number] }");
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_extern_tuple_single_single_field_single_variant() {
    test_case! {
        enum Test {
            A(usize),
        }
    }
    let json = serde_json::to_value(Test::A(123)).unwrap();
    assert_eq!(json, serde_json::json!({"A": 123}));

    let number_schema = usize::schema();
    assert_eq!(Test::schema(), format!("z.object({{A: {number_schema}}})"));
    assert_eq!(Test::type_def(), "{ A: number }");
    assert_eq!(Test::type_name(), "Ns.Test");
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

    assert_eq!(
        Test::schema(),
        format!("z.union([z.literal(\"A\"), z.literal(\"B\")])")
    );
    assert_eq!(Test::type_def(), "\"A\" | \"B\"");
    assert_eq!(Test::type_name(), "Ns.Test");
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

    assert_eq!(Test::schema(), format!("z.literal(\"A\")"));
    assert_eq!(Test::type_def(), "\"A\"");
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_intern_struct() {
    test_case! {
        #[serde(tag = "type")]
        enum Test {
            A { s: String },
            B { num: usize },
        }
    }
    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"type": "B", "num": 123}));

    let string_schema = String::schema();
    let number_schema = usize::schema();

    assert_eq!(
    Test::schema(),
    format!("z.discriminatedUnion(\"type\", [z.object({{ type: z.literal(\"A\"), s: {string_schema} }}), z.object({{ type: z.literal(\"B\"), num: {number_schema} }})])")
    );
    assert_eq!(
        Test::type_def(),
        "{ type: \"A\", s: string } | { type: \"B\", num: number }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_intern_struct_multiple_fields() {
    test_case! {
        #[serde(tag = "type")]
        enum Test {
            A { s: String, num: usize },
            B { num: usize },
        }
    }

    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"type": "B", "num": 123}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.discriminatedUnion(\"type\", [z.object({{ type: z.literal(\"A\"), s: {string_schema}, num: {number_schema} }}), z.object({{ type: z.literal(\"B\"), num: {number_schema} }})])")
    );
    assert_eq!(
        Test::type_def(),
        "{ type: \"A\", s: string, num: number } | { type: \"B\", num: number }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_intern_struct_multiple_fields_single_variant() {
    test_case! {
        #[serde(tag = "type")]
        enum Test {
            A { s: String, num: usize },
        }
    }

    let json = serde_json::to_value(Test::A {
        s: String::new(),
        num: 123,
    })
    .unwrap();
    assert_eq!(json, serde_json::json!({"type": "A", "s": "", "num": 123}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.object({{ type: z.literal(\"A\"), s: {string_schema}, num: {number_schema} }})")
    );
    assert_eq!(Test::type_def(), "{ type: \"A\", s: string, num: number }");
    assert_eq!(Test::type_name(), "Ns.Test");
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

    assert_eq!(
        Test::schema(),
        format!(
            "z.discriminatedUnion(\"type\", [z.object({{ type: z.literal(\"A\") }}), z.object({{ type: z.literal(\"B\") }})])"
        )
    );
    assert_eq!(Test::type_def(), "{ type: \"A\" } | { type: \"B\" }");
    assert_eq!(Test::type_name(), "Ns.Test");
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

    assert_eq!(
        Test::schema(),
        format!("z.object({{ type: z.literal(\"A\") }})")
    );
    assert_eq!(Test::type_def(), "{ type: \"A\" }");
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_untagged_struct() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A { s: String },
            B { num: usize },
        }
    }
    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"num": 123}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!(
            "z.union([z.object({{ s: {string_schema} }}), z.object({{ num: {number_schema} }})])"
        )
    );
    assert_eq!(Test::type_def(), "{ s: string } | { num: number }");
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_untagged_struct_multiple_fields() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A { s: String, num: usize },
            B { num: usize },
        }
    }
    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"num": 123}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.union([z.object({{ s: {string_schema}, num: {number_schema} }}), z.object({{ num: {number_schema} }})])")
    );
    assert_eq!(
        Test::type_def(),
        "{ s: string, num: number } | { num: number }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_untagged_struct_multiple_fields_single_variant() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A { s: String, num: usize },
        }
    }
    let json = serde_json::to_value(Test::A {
        s: String::new(),
        num: 123,
    })
    .unwrap();
    assert_eq!(json, serde_json::json!({"s": "", "num": 123}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.object({{ s: {string_schema}, num: {number_schema} }})")
    );
    assert_eq!(Test::type_def(), "{ s: string, num: number }");
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_untagged_tuple() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A(String),
            B(usize),
        }
    }
    let json = serde_json::to_value(Test::B(123)).unwrap();
    assert_eq!(json, serde_json::json!(123));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.union([{string_schema}, {number_schema}])")
    );
    assert_eq!(Test::type_def(), "string | number");
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_untagged_tuple_multiple_fields() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A(usize, usize),
            B(String),
        }
    }
    let json = serde_json::to_value(Test::A(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!([123, 42]));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.union([z.tuple([{number_schema}, {number_schema}]), {string_schema}])")
    );
    assert_eq!(Test::type_def(), "[number, number] | string");
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_untagged_tuple_multiple_fields_single_variant() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A(usize, usize),
        }
    }

    let json = serde_json::to_value(Test::A(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!([123, 42]));

    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.tuple([{number_schema}, {number_schema}])")
    );
    assert_eq!(Test::type_def(), "[number, number]");
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn enum_untagged_tuple_single_field_single_variant() {
    test_case! {
        #[serde(untagged)]
        enum Test {
            A(usize),
        }
    }
    let json = serde_json::to_value(Test::A(123)).unwrap();
    assert_eq!(json, serde_json::json!(123));

    let number_schema = usize::schema();
    assert_eq!(Test::schema(), format!("{number_schema}"));
    assert_eq!(Test::type_def(), "number");
    assert_eq!(Test::type_name(), "Ns.Test");
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

    assert_eq!(Test::schema(), format!("z.union([z.null(), z.null()])"));
    assert_eq!(Test::type_def(), "null | null");
    assert_eq!(Test::type_name(), "Ns.Test");
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

    assert_eq!(Test::schema(), String::from("z.null()"));
    assert_eq!(Test::type_def(), "null");
    assert_eq!(Test::type_name(), "Ns.Test");
}
