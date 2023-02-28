use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

mod test_utils;

#[test]
fn serde_skip_struct_field() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Test {
            #[serde(skip)]
            to_be_skipped: String,
            num: usize,
        }
    }

    let value = Test {
        to_be_skipped: String::new(),
        num: 123,
    };

    assert_eq!(
        value,
        serde_json::from_value(serde_json::json!({"num": 123})).unwrap()
    );
    assert!(!Test::schema().contains("to_be_skipped"));
}

#[test]
fn serde_skip_deserializing_struct_field() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Test {
            #[serde(skip_deserializing)]
            to_be_skipped: String,
            num: usize,
        }
    }

    let value = Test {
        to_be_skipped: String::new(),
        num: 123,
    };

    assert_eq!(
        value,
        serde_json::from_value(serde_json::json!({"num": 123})).unwrap()
    );

    assert!(!Test::schema().contains("to_be_skipped"));
}
