use zod::ZodType;

mod test_utils;
use test_utils::*;

fn main() {}

#[test]
fn ok() {
    test_case! {
        struct Test(usize);
    }

    let json = serde_json::to_string(&Test(123)).unwrap();
    assert_eq!(json, "123");

    compare(
        Test::AST.to_zod_string(),
        "export const Test = z.lazy(() => Rs.Usize);",
    );
    compare(Test::AST.to_ts_string(), "export type Test = Rs.Usize;")
}
