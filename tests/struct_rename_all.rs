use pretty_assertions::assert_eq;
use zod::ZodType;

mod test_utils;
use test_utils::*;

fn main() {}

#[test]
fn rename_all_struct() {
    test_case! {
        #[serde(rename_all = "UPPERCASE")]
        struct Test {
           string_value: String,
           usize_value: usize
        }
    }

    let json = serde_json::to_value(Test {
        string_value: String::from("abc"),
        usize_value: 123,
    })
    .unwrap();
    assert_eq!(
        json,
        serde_json::json!({"STRING_VALUE": "abc", "USIZE_VALUE": 123})
    );

    assert!(Test::AST.to_zod_string().contains("USIZE_VALUE"),);
    assert!(Test::AST.to_zod_string().contains("STRING_VALUE"),);

    assert!(Test::AST.to_ts_string().contains("USIZE_VALUE"),);
    assert!(Test::AST.to_ts_string().contains("STRING_VALUE"),);
}
