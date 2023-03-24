mod test_utils;
use test_utils::*;

#[test]
fn simple_named_struct() {
    test_case! {
        #[derive(serde::Deserialize)]
        struct Test {
            s: String,
            num: Usize,
        }
    }

    compare_export::<Test>(
        "export const Test = z.lazy(() => z.object({ s: Rs.String, num: Rs.Usize }));",
        "export interface Test { s: Rs.String, num: Rs.Usize }",
    );
}

#[test]
fn simple_tuple_struct() {
    test_case! {
        #[derive(serde::Deserialize)]
        struct Test(String);
    }

    compare_export::<Test>(
        "export const Test = z.lazy(() => z.tuple([Rs.String]));",
        "export type Test = [Rs.String];",
    );
}
