mod test_utils;
use test_utils::*;

#[test]
fn ok() {
    test_case! {
        struct Test(Usize);
    }

    let json = serde_json::to_string(&Test(Usize(123))).unwrap();
    assert_eq!(json, "\"123\"");

    compare_export::<Test>(
        "export const Test = z.lazy(() => Rs.Usize);",
        "export type Test = Rs.Usize;",
    );
}
