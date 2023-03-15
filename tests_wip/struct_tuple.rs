use zod::ZodType;

mod test_utils;
use test_utils::*;

fn main() {}

#[test]
fn ok() {
    test_case! {
        struct Test(usize, usize, String);
    }

    let json = serde_json::to_value(Test(123, 42, String::from("abc"))).unwrap();
    assert_eq!(json, serde_json::json!([123, 42, "abc"]));

    assert_eq!(
        Test::schema(),
        tuple(&[usize::schema(), usize::schema(), String::schema()])
    );
    assert_eq!(Test::type_def(), "[number, number, string]");
    assert_eq!(Test::inline().to_string(), "Ns.Test")
}

#[test]
fn with_default_fields() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Test(usize, #[serde(default)]usize);
    }

    let test = Test(42, 0);

    let res: Test = serde_json::from_value(serde_json::json!([42])).unwrap();

    assert_eq!(test, res);

    assert_eq!(
        Test::schema(),
        tuple(&[usize::schema(), optional(usize::schema())])
    );

    assert_eq!(Test::type_def(), "[number, number | undefined]");
    assert_eq!(Test::inline().to_string(), "Ns.Test")
}
