use zod::ZodType;

mod test_utils;
use test_utils::*;

fn main() {}

#[test]
fn one_tuple() {
    test_case! {
        struct Test(usize);
    }

    let json = serde_json::to_value(Test(123)).unwrap();
    assert_eq!(json, serde_json::json!(123));

    compare(
        Test::AST.to_zod_string(),
        "export const Test = z.lazy(() => Rs.Usize);",
    );

    compare(Test::AST.to_ts_string(), "export type Test = Rs.Usize;");
}

#[test]
fn ok() {
    test_case! {
        struct Test(usize, usize, String);
    }

    let json = serde_json::to_value(Test(123, 42, String::from("abc"))).unwrap();
    assert_eq!(json, serde_json::json!([123, 42, "abc"]));

    compare(
        Test::AST.to_zod_string(),
        "export const Test = z.lazy(() => z.tuple([Rs.Usize, Rs.Usize, Rs.String]));",
    );
    compare(
        Test::AST.to_ts_string(),
        "export type Test = [Rs.Usize, Rs.Usize, Rs.String];",
    );
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

    compare(
        Test::AST.to_zod_string(),
        "export const Test = z.lazy(() => z.tuple([Rs.Usize, Rs.Usize.optional()]));",
    );

    assert_eq!(
        Test::AST.to_ts_string(),
        "export type Test = [Rs.Usize, Rs.Usize | undefined];"
    );
}
