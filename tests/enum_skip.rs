mod test_utils;
use test_utils::*;

#[test]
fn serde_skip_enum_tuple_variant() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            #[serde(skip)]
            ToBeSkipped(String),
            Number(Usize),
        }
    }

    let export = <Test as RequestType>::EXPORT;

    assert!(!export.to_zod_string().contains("ToBeSkipped"));
}

#[test]
fn serde_skip_enum_unit_variant() {
    test_case! {
    #[derive(Debug, PartialEq, serde::Deserialize)]
    enum Test {
        #[serde(skip)]
        ToBeSkipped,
            Number(u16),
        }
    }

    let export = Test::EXPORT;

    assert!(!export.to_zod_string().contains("ToBeSkipped"));
    assert!(!export.to_ts_string().contains("ToBeSkipped"));
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
            Number(u16),
        }
    }

    let export = Test::EXPORT;

    assert!(!export.to_zod_string().contains("ToBeSkipped"));
    assert!(!export.to_ts_string().contains("ToBeSkipped"));
}

#[test]
fn serde_skip_enum_tuple_field() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            A(#[serde(skip)] String, u16),
            B(u16),
        }
    }

    let json = serde_json::to_value(Test::A(String::new(), 123)).unwrap();
    assert_eq!(json, serde_json::json!({"A": [123]}));

    let export = Test::EXPORT;

    assert!(!export.to_zod_string().contains("z.string"));
    assert!(!export.to_ts_string().contains("string"));
}

#[test]
fn serde_skip_enum_only_tuple_field() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            A,
            B(u16),
        }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize, zod::RequestType)]
    #[zod(namespace = "Ns")]
    enum Test2 {
        A(#[serde(skip)] String),
        B(u16),
    }

    let json = serde_json::to_value(Test2::A(String::new())).unwrap();
    assert_eq!(json, serde_json::json!("A"));

    assert_eq!(Test::EXPORT.schema, Test2::EXPORT.schema);
}

#[test]
fn serde_skip_struct_field() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            A {
                #[serde(skip)]
                s: String,
                num: u16
            },
            B(u16),
        }
    }

    #[derive(Debug, PartialEq, serde::Deserialize, zod::RequestType)]
    #[zod(namespace = "Ns")]
    enum TestExpected {
        A { num: u16 },
        B(u16),
    }

    assert_eq!(Test::EXPORT.schema, TestExpected::EXPORT.schema);
}
