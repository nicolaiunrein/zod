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

    compare_export::<Test>(
        "export const Test = z.lazy(() => Rs.String);",
        "export type Test = Rs.String;",
    );
}

#[test]
fn serde_transparent_newtype_struct() {
    test_case! {
        #[derive(serde::Deserialize)]
        #[serde(transparent)]
        struct Test(String);
    }

    compare_export::<Test>(
        "export const Test = z.lazy(() => Rs.String);",
        "export type Test = Rs.String;",
    );
}
