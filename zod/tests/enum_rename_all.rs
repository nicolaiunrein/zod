use pretty_assertions::assert_eq;
use zod::{zod, Codegen, Namespace};

mod test_utils;
// use test_utils::test_case;

fn main() {}

#[test]
fn rename_all_struct() {
    test_case! {
        #[serde(rename_all = "snake_case")]
        enum Test {
            HelloWorld { s: String },
            AnotherValue { num: usize },
        }
    }

    let json = serde_json::to_value(Test::AnotherValue { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"another_value": {"num": 123}}));

    assert!(Test::schema().contains("hello_world"),);
    assert!(Test::schema().contains("another_value"),);

    assert!(Test::type_def().contains("hello_world"),);
    assert!(Test::type_def().contains("another_value"),);

    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn rename_all_tuple() {
    test_case! {
        #[serde(rename_all = "snake_case")]
        enum Test {
            HelloWorld(String, usize),
            AnotherValue(usize, usize)
        }
    }

    let json = serde_json::to_value(Test::AnotherValue(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!({"another_value": [123, 42]}));

    assert!(Test::schema().contains("hello_world"),);
    assert!(Test::schema().contains("another_value"),);

    assert!(Test::type_def().contains("hello_world"),);
    assert!(Test::type_def().contains("another_value"),);

    assert_eq!(Test::type_name(), "Ns.Test");
}

#[test]
fn rename_all_unit() {
    test_case! {
        #[serde(rename_all = "snake_case")]
        enum Test {
            HelloWorld,
            AnotherValue
        }
    }

    let json = serde_json::to_value(Test::AnotherValue).unwrap();
    assert_eq!(json, serde_json::json!("another_value"));

    assert!(Test::schema().contains("hello_world"),);
    assert!(Test::schema().contains("another_value"),);

    assert!(Test::type_def().contains("hello_world"),);
    assert!(Test::type_def().contains("another_value"),);

    assert_eq!(Test::type_name(), "Ns.Test");
}
