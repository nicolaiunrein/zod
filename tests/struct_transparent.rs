use zod::ZodType;
mod test_utils;
use test_utils::*;

#[test]
fn serde_transparent_struct() {
    test_case! {
        #[derive(serde::Deserialize)]
        #[serde(transparent)]
        struct Test {
            s: String,
        }
    }

    compare(
        Test::AST.to_zod_string(),
        "export const Test = z.lazy(() => Rs.String);",
    );
    compare(Test::AST.to_ts_string(), "export type Test = Rs.String;");
}

#[test]
fn serde_transparent_newtype_struct() {
    test_case! {
        #[derive(serde::Deserialize)]
        #[serde(transparent)]
        struct Test(String);
    }

    compare(
        Test::AST.to_zod_string(),
        "export const Test = z.lazy(() => Rs.String);",
    );
    compare(Test::AST.to_ts_string(), "export type Test = Rs.String;");
}
