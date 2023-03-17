use pretty_assertions::assert_eq;
use zod::ZodType;

mod test_utils;
use test_utils::*;

fn main() {}

#[test]
fn rename_all_struct() {
    test_case! {
        struct Test {
            #[serde(rename = "after")]
           before: String,
           usize_value: usize
        }
    }

    let json = serde_json::to_value(Test {
        before: String::from("abc"),
        usize_value: 123,
    })
    .unwrap();
    assert_eq!(
        json,
        serde_json::json!({"after": "abc", "usize_value": 123})
    );

    assert!(Test::AST.to_zod_string().contains("after"),);
    assert!(!Test::AST.to_zod_string().contains("before"),);

    assert!(Test::AST.to_ts_string().contains("after"),);
    assert!(!Test::AST.to_ts_string().contains("before"),);
}
