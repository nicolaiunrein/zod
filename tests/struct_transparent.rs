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
        Test::CODE.schema,
        "export const Test = z.lazy(() => Rs.String);",
    );
    compare(Test::CODE.type_def, "export type Test = Rs.String;");
}

#[test]
fn serde_transparent_newtype_struct() {
    test_case! {
    #[derive(serde::Deserialize)]
    #[serde(transparent)]
    struct Test(String);
    }

    compare(
        Test::CODE.schema,
        "export const Test = z.lazy(() => Rs.String);",
    );
    compare(Test::CODE.type_def, "export type Test = Rs.String;");
}
