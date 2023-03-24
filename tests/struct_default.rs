mod test_utils;
use test_utils::*;
use zod::types::Usize;

#[test]
fn serde_default_named_struct_field() {
    test_case! {
        #[derive(serde::Deserialize)]
        struct Test {
            #[serde(default)]
            s: String,
            num: Usize,
        }
    }

    compare_export::<Test>(
        "export const Test = z.lazy(() => z.object({s: Rs.String.optional(), num: Rs.Usize}));",
        "export interface Test { s?: Rs.String | undefined, num: Rs.Usize }",
    );
}

#[ignore]
#[test]
fn serde_default_tuple_struct_field() {
    test_case! {
    #[derive(serde::Deserialize)]
    struct Test(#[serde(default)] String);
    }

    compare_export::<Test>(
        "export const Test = z.lazy(() => Rs.String.optional());",
        "export type Test = Rs.String | undefined;",
    );
}
