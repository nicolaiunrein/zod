mod test_utils;
use test_utils::*;

#[test]
fn struct_default_named() {
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

#[test]
fn struct_default_tuple() {
    test_case! {
    #[derive(serde::Deserialize)]
    struct Test(#[serde(default)] String);
    }

    compare_inlined::<Test>(
        "export const Test = z.lazy(() => Rs.String.optional());",
        "export type Test = Rs.String | undefined;",
    );
}
