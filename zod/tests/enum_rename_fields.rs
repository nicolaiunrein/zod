use pretty_assertions::assert_eq;
use zod::{zod, Codegen, Namespace};

mod test_utils;

fn main() {}

#[test]
fn rename_variant_struct() {
    test_case! {
        enum Test {
            HelloWorld { s: String },
            #[serde(rename = "after")]
            AnotherValue { num: usize },
        }
    }

    let json = serde_json::to_value(Test::AnotherValue { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"after": {"num": 123}}));

    assert!(Test::schema().contains("HelloWorld"),);
    assert!(Test::schema().contains("after"));
    assert!(!Test::schema().contains("AnotherValue"));

    assert!(Test::type_def().contains("HelloWorld"),);
    assert!(Test::type_def().contains("after"));
    assert!(!Test::type_def().contains("AnotherValue"));

    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn rename_struct_field() {
    test_case! {
        enum Test {
            HelloWorld { s: String },
            AnotherValue {
                #[serde(rename = "after")]
                before: usize
            },
        }
    }

    let json = serde_json::to_value(Test::AnotherValue { before: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"AnotherValue": {"after": 123}}));

    assert!(Test::schema().contains("HelloWorld"),);
    assert!(Test::schema().contains("AnotherValue"),);
    assert!(Test::schema().contains("after"));
    assert!(!Test::schema().contains("before"));

    assert!(Test::type_def().contains("HelloWorld"),);
    assert!(Test::type_def().contains("AnotherValue"),);
    assert!(Test::type_def().contains("after"));
    assert!(!Test::type_def().contains("before"));
    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn rename_all_tuple() {
    test_case! {
    enum Test {
        HelloWorld(String, usize),
        #[serde(rename = "after")]
        AnotherValue(usize, usize)
        }
    }

    let json = serde_json::to_value(Test::AnotherValue(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!({"after": [123, 42]}));

    assert!(Test::schema().contains("HelloWorld"),);
    assert!(Test::schema().contains("after"));
    assert!(!Test::schema().contains("before"));

    assert!(Test::type_def().contains("HelloWorld"),);
    assert!(Test::type_def().contains("after"),);
    assert!(!Test::type_def().contains("before"));

    assert_eq!(Test::type_name(), "Ns.Test");
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

    assert!(Test::schema().contains("HelloWorld"),);
    assert!(Test::schema().contains("after"));
    assert!(!Test::schema().contains("before"));

    assert!(Test::type_def().contains("HelloWorld"),);
    assert!(Test::type_def().contains("after"));
    assert!(!Test::type_def().contains("before"));

    assert_eq!(Test::type_name(), "Ns.Test");
}
