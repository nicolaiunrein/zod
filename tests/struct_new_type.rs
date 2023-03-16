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
        Test::AST.schema,
        "export const Test = z.lazy(() => Rs.Usize);",
    );
    compare(Test::AST.type_def, "export type Test = Rs.Usize;")
}
