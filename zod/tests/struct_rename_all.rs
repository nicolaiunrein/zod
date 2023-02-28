use pretty_assertions::assert_eq;
use zod::{Codegen, Namespace};

mod test_utils;

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

    assert!(Test::schema().contains("USIZE_VALUE"),);
    assert!(Test::schema().contains("STRING_VALUE"),);

    assert!(Test::type_def().contains("USIZE_VALUE"),);
    assert!(Test::type_def().contains("STRING_VALUE"),);

    assert_eq!(Test::type_name(), "Ns.Test");
}
