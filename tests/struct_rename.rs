use pretty_assertions::assert_eq;
use zod::ZodType;

mod test_utils;
use test_utils::*;

#[test]
fn serde_name_named_struct() {
    test_case! {
        #[serde(rename= "Hello")]
        struct Test {
            s: String,
            num: usize,
        }
    }

    assert!(Test::AST.to_zod_string().starts_with("export const Hello"));
    assert!(Test::AST
        .to_ts_string()
        .starts_with("export interface Hello"));
    compare(
        Test::AST.to_ts_string(),
        "export interface Hello { s: Rs.String, num: Rs.Usize }",
    )
}

#[test]
fn serde_name_tuple_str() {
    test_case! {
    #[serde(rename= "HelloTuple")]
    struct Test(String);
    }
    assert!(Test::AST
        .to_zod_string()
        .starts_with("export const HelloTuple"));
    compare(
        Test::AST.to_ts_string(),
        "export type HelloTuple = Rs.String;",
    );
}

#[test]
fn serde_rename_struct_field() {
    test_case! {
    struct Test {
    #[serde(rename= "after")]
    before: String,
    other: usize,
    }
    }

    assert!(Test::AST.to_zod_string().contains("after"));
    assert!(Test::AST.to_zod_string().contains("other"));
    assert!(!Test::AST.to_zod_string().contains("before"));
}
