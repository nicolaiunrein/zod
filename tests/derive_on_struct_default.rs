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
        struct Test(String, #[serde(default)] Usize);
    }

    compare_export::<Test>(
        "export const Test = z.lazy(() => z.tuple([Rs.String, Rs.Usize.optional()]));",
        "export type Test = [Rs.String, Rs.Usize | undefined];",
    );
}

#[test]
fn struct_default_newtype() {
    test_case! {
        struct Test(#[serde(default)]Usize);
    }

    let json = serde_json::to_string(&Test(Usize(123))).unwrap();
    assert_eq!(json, "\"123\"");

    compare_export::<Test>(
        "export const Test = z.lazy(() => Rs.Usize.optional());",
        "export type Test = Rs.Usize | undefined;",
    );
}