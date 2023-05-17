mod test_utils;
use test_utils::*;

#[test]
fn ok() {
    test_case! {
        struct Test(Usize, Usize, String);
    }

    let json = serde_json::to_value(Test(Usize(123), Usize(42), String::from("abc"))).unwrap();
    assert_eq!(json, serde_json::json!(["123", "42", "abc"]));

    compare_export::<Test>(
        "export const Test = z.lazy(() => z.tuple([Rs.Usize, Rs.Usize, Rs.String]));",
        "export type Test = [Rs.Usize, Rs.Usize, Rs.String];",
    );
}

#[test]
fn with_default_fields() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Test(Usize, #[serde(default)]Usize);
    }

    let test = Test(Usize(42), Usize(0));

    let res: Test = serde_json::from_value(serde_json::json!(["42"])).unwrap();

    assert_eq!(test, res);

    compare_export::<Test>(
        "export const Test = z.lazy(() => z.tuple([Rs.Usize, Rs.Usize.optional()]));",
        "export type Test = [Rs.Usize, Rs.Usize | undefined];",
    );
}
