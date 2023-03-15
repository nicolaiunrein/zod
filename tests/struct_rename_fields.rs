use pretty_assertions::assert_eq;
use zod::ZodType;

mod test_utils;

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

    assert!(Test::CODE.schema.contains("after"),);
    assert!(!Test::CODE.schema.contains("before"),);

    assert!(Test::CODE.type_def.contains("after"),);
    assert!(!Test::CODE.type_def.contains("before"),);
}
