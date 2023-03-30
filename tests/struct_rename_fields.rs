use pretty_assertions::assert_eq;

mod test_utils;
use test_utils::*;

fn main() {}

#[test]
fn rename_all_struct() {
    test_case! {
        struct Test {
            #[serde(rename = "after")]
           before: String,
           usize_value: Usize
        }
    }

    let json = serde_json::to_value(Test {
        before: String::from("abc"),
        usize_value: Usize(123),
    })
    .unwrap();
    assert_eq!(
        json,
        serde_json::json!({"after": "abc", "usize_value": "123"})
    );

    assert!(Test::export().to_zod_string().contains("after"),);
    assert!(!Test::export().to_zod_string().contains("before"),);

    assert!(Test::export().to_ts_string().contains("after"),);
    assert!(!Test::export().to_ts_string().contains("before"),);
}
