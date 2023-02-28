#![cfg(test)]
use pretty_assertions::assert_eq;
use zod::{Codegen, Namespace};

mod test_utils;
use test_utils::*;

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

    assert_eq!(
        Test::schema(),
        discriminated_union(
            "type",
            &[
                adj_tagged("A", object! { s : String::schema() }),
                adj_tagged("B", object! { num : usize::schema() }),
            ],
        )
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

    assert_eq!(
        Test::schema(),
        discriminated_union(
            "type",
            &[
                adj_tagged(
                    "A",
                    object! {
                        s: String::schema(),
                        num: usize::schema()
                    }
                ),
                adj_tagged("B", object! { num: usize::schema() }),
            ]
        )
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

    assert_eq!(
        Test::schema(),
        adj_tagged("A", object! { s: String::schema(), num: usize::schema() })
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

    assert_eq!(
        Test::schema(),
        discriminated_union(
            "type",
            &[
                adj_tagged("A", String::schema()),
                adj_tagged("B", usize::schema())
            ]
        )
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

    assert_eq!(
        Test::schema(),
        discriminated_union(
            "type",
            &[
                adj_tagged("A", tuple(&[&usize::schema(), &usize::schema()])),
                adj_tagged("B", String::schema())
            ]
        )
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

    assert_eq!(
        Test::schema(),
        adj_tagged("A", tuple(&[&usize::schema(), &usize::schema()]))
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

    assert_eq!(Test::schema(), adj_tagged("A", &usize::schema()));
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
        discriminated_union("type", &[object! {type: A}, object! {type: B}])
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

    assert_eq!(Test::schema(), object! { type: A });
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

    assert_eq!(
        Test::schema(),
        zod_union(&[
            object! { A: object! { s: String::schema() }},
            object! { B: object! { num: usize::schema() }}
        ])
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

    assert_eq!(
        Test::schema(),
        zod_union(&[
            object! {
                A: object! {
                    s: String::schema(),
                    num: usize::schema()
                }
            },
            object! {
                B: object! {
                    num: usize::schema()
                }
            }
        ])
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

    assert_eq!(
        Test::schema(),
        object! {
            A: object! {
                s: String::schema(),
                num: usize::schema()
            }
        }
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

    assert_eq!(
        Test::schema(),
        zod_union(&[
            object! {
                A: String::schema()
            },
            object! {
                B: usize::schema()
            }
        ])
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

    assert_eq!(
        Test::schema(),
        zod_union(&[
            object! {
                A: tuple(&[usize::schema(), usize::schema()])
            },
            object! {
                B: String::schema()
            }
        ])
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

    assert_eq!(
        Test::schema(),
        object! {
            A: tuple(&[usize::schema(), usize::schema()])
        }
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

    assert_eq!(
        Test::schema(),
        object! {
            A: usize::schema()
        }
    );
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

    assert_eq!(Test::schema(), zod_union(&[A, B]));
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

    assert_eq!(Test::schema(), A);
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

    assert_eq!(
        Test::schema(),
        discriminated_union(
            "type",
            &[
                object! {
                    type: A,
                    s: &String::schema()
                },
                object! {
                    type: B,
                    num: &usize::schema()
                }
            ]
        )
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

    assert_eq!(
        Test::schema(),
        discriminated_union(
            "type",
            &[
                object! {
                    type: A,
                    s: &String::schema(),
                    num: &usize::schema()
                },
                object! {
                    type: B,
                    num: &usize::schema()
                }
            ]
        )
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

    assert_eq!(
        Test::schema(),
        object! {
            type: A,
            s: &String::schema(),
            num: &usize::schema()
        }
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
        discriminated_union("type", &[object! { type: A }, object! { type: B }])
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

    assert_eq!(Test::schema(), object! { type: A });
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

    assert_eq!(
        Test::schema(),
        zod_union(&[
            object! { s: String::schema() },
            object! { num: usize::schema() }
        ])
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

    assert_eq!(
        Test::schema(),
        zod_union(&[
            object! { s: String::schema(), num: usize::schema() },
            object! { num: usize::schema() }
        ])
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

    assert_eq!(
        Test::schema(),
        object! { s: String::schema(), num: usize::schema() }
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

    assert_eq!(
        Test::schema(),
        zod_union(&[String::schema(), usize::schema()])
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

    assert_eq!(
        Test::schema(),
        zod_union(&[tuple(&[usize::schema(), usize::schema()]), String::schema()])
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
    assert_eq!(Test::schema(), tuple(&[usize::schema(), usize::schema()]));
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
    assert_eq!(Test::schema(), usize::schema());
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
    assert_eq!(Test::schema(), zod_union(&[NULL, NULL]));
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
    assert_eq!(Test::schema(), NULL);
    assert_eq!(Test::type_def(), "null");
    assert_eq!(Test::type_name(), "Ns.Test");
}
