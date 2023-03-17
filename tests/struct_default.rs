use zod::ast::{FormatTypescript, FormatZod};
use zod::ZodType;

mod test_utils;
use test_utils::*;

#[test]
fn serde_default_named_struct_field() {
    test_case! {
        #[derive(serde::Deserialize)]
        struct Test {
            #[serde(default)]
            s: String,
            num: usize,
        }
    }

    compare(
        Test::AST.to_zod_string(),
        "export const Test = z.lazy(() => z.object({s: Rs.String.optional(), num: Rs.Usize}));",
    );
    compare(
        Test::AST.to_ts_string(),
        "export interface Test { s?: Rs.String | undefined, num: Rs.Usize }",
    )
}

#[test]
fn serde_default_tuple_struct_field() {
    test_case! {
        #[derive(serde::Deserialize)]
        struct Test(#[serde(default)] String);
    }

    compare(
        Test::AST.to_zod_string(),
        "export const Test = z.lazy(() => Rs.String.optional());",
    );
    compare(
        Test::AST.to_ts_string(),
        "export type Test = Rs.String | undefined;",
    )
}
