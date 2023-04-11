mod test_utils;
use test_utils::*;

fn main() {}

#[test]
fn rename_variant_struct() {
    test_case! {
        enum Test {
            HelloWorld { s: String },
            #[serde(rename = "after")]
            AnotherValue { num: u16 },
        }
    }

    let json = serde_json::to_value(Test::AnotherValue { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"after": {"num": 123}}));

    let export = <Test as zod::RequestType>::EXPORT;

    assert!(export.to_zod_string().contains("HelloWorld"),);
    assert!(export.to_zod_string().contains("after"));
    assert!(!export.to_zod_string().contains("AnotherValue"));

    assert!(export.to_ts_string().contains("HelloWorld"),);
    assert!(export.to_ts_string().contains("after"));
    assert!(!export.to_ts_string().contains("AnotherValue"));
}

#[test]
fn rename_struct_field() {
    test_case! {
        enum Test {
            HelloWorld { s: String },
            AnotherValue {
                #[serde(rename = "after")]
                before: u16
            },
        }
    }

    let json = serde_json::to_value(Test::AnotherValue { before: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"AnotherValue": {"after": 123}}));

    let export = <Test as zod::RequestType>::EXPORT;

    assert!(export.to_zod_string().contains("HelloWorld"),);
    assert!(export.to_zod_string().contains("AnotherValue"),);
    assert!(export.to_zod_string().contains("after"));
    assert!(!export.to_zod_string().contains("before"));

    assert!(export.to_ts_string().contains("HelloWorld"),);
    assert!(export.to_ts_string().contains("AnotherValue"),);
    assert!(export.to_ts_string().contains("after"));
    assert!(!export.to_ts_string().contains("before"));
}

#[test]
fn rename_all_tuple() {
    test_case! {
    enum Test {
        HelloWorld(String, u16),
        #[serde(rename = "after")]
        AnotherValue(u16, u16)
        }
    }

    let json = serde_json::to_value(Test::AnotherValue(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!({"after": [123, 42]}));

    let export = <Test as zod::RequestType>::EXPORT;

    assert!(export.to_zod_string().contains("HelloWorld"),);
    assert!(export.to_zod_string().contains("after"));
    assert!(!export.to_zod_string().contains("before"));

    assert!(export.to_ts_string().contains("HelloWorld"),);
    assert!(export.to_ts_string().contains("after"),);
    assert!(!export.to_ts_string().contains("before"));
}

#[test]
fn rename_all_unit() {
    test_case! {
    enum Test {
        HelloWorld,
        #[serde(rename = "after")]
        AnotherValue
        }
    }

    let json = serde_json::to_value(Test::AnotherValue).unwrap();
    assert_eq!(json, serde_json::json!("after"));

    let export = <Test as zod::RequestType>::EXPORT;

    assert!(export.to_zod_string().contains("HelloWorld"),);
    assert!(export.to_zod_string().contains("after"));
    assert!(!export.to_zod_string().contains("before"));

    assert!(export.to_ts_string().contains("HelloWorld"),);
    assert!(export.to_ts_string().contains("after"));
    assert!(!export.to_ts_string().contains("before"));
}
