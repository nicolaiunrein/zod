use zod::ZodType;

mod test_utils;
// use test_utils::test_case;

#[test]
fn serde_skip_enum_tuple_variant() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            #[serde(skip)]
            ToBeSkipped(String),
            Number(usize),
        }
    }

    assert!(!Test::schema().contains("ToBeSkipped"));
}

#[test]
fn serde_skip_enum_unit_variant() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            #[serde(skip)]
            ToBeSkipped,
            Number(usize),
        }
    }

    assert!(!Test::schema().contains("ToBeSkipped"));
}

#[test]
fn serde_skip_enum_struct_variant() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            #[serde(skip)]
            ToBeSkipped {
                s: String
            },
            Number(usize),
        }
    }

    assert!(!Test::schema().contains("ToBeSkipped"));
}

#[test]
fn serde_skip_enum_tuple_field() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            A(#[serde(skip)] String, usize),
            B(usize),
        }
    }

    let json = serde_json::to_value(Test::A(String::new(), 123)).unwrap();
    assert_eq!(json, serde_json::json!({"A": [123]}));

    assert!(!Test::schema().contains("z.string"));
}

#[test]
fn serde_skip_enum_only_tuple_field() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
            enum Test {
            A,
            B(usize),
        }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, zod::Zod)]
    #[zod(namespace = "Ns")]
    enum Test2 {
        A(#[serde(skip)] String),
        B(usize),
    }

    let json = serde_json::to_value(Test2::A(String::new())).unwrap();
    assert_eq!(json, serde_json::json!("A"));

    assert_eq!(Test::schema(), Test2::schema());
    assert_eq!(Test::type_def(), Test2::type_def());
}

#[test]
fn serde_skip_struct_field() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            A {
                #[serde(skip)]
                s: String,
                num: usize
            },
            B(usize),
        }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, zod::Zod)]
    #[zod(namespace = "Ns")]
    enum TestExpected {
        A { num: usize },
        B(usize),
    }

    assert_eq!(Test::schema(), TestExpected::schema());
    assert_eq!(Test::type_def(), TestExpected::type_def());
}
