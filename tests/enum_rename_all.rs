mod test_utils;
use test_utils::*;

fn main() {}

#[test]
fn rename_all_struct() {
    test_case! {
        #[serde(rename_all = "snake_case")]
        enum Test {
            HelloWorld { s: String },
            AnotherValue { num: Usize },
        }
    }

    let json = serde_json::to_value(Test::AnotherValue { num: Usize(123) }).unwrap();
    assert_eq!(json, serde_json::json!({"another_value": {"num": "123"}}));

    let export = <Test as zod::RequestType>::EXPORT;

    assert!(export.to_zod_string().contains("hello_world"),);
    assert!(export.to_zod_string().contains("another_value"),);

    assert!(export.to_ts_string().contains("hello_world"),);
    assert!(export.to_ts_string().contains("another_value"),);
}

#[test]
fn rename_all_tuple() {
    test_case! {
        #[serde(rename_all = "snake_case")]
        enum Test {
            HelloWorld(String, u16),
            AnotherValue(u16, u16)
        }
    }

    let json = serde_json::to_value(Test::AnotherValue(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!({"another_value": [123, 42]}));

    let export = <Test as zod::RequestType>::EXPORT;

    assert!(export.to_zod_string().contains("hello_world"),);
    assert!(export.to_zod_string().contains("another_value"),);

    assert!(export.to_ts_string().contains("hello_world"),);
    assert!(export.to_ts_string().contains("another_value"),);
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

    let export = <Test as zod::RequestType>::EXPORT;

    assert!(export.to_zod_string().contains("hello_world"),);
    assert!(export.to_zod_string().contains("another_value"),);

    assert!(export.to_ts_string().contains("hello_world"),);
    assert!(export.to_ts_string().contains("another_value"),);
}
