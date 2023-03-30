mod test_utils;
use test_utils::*;

#[test]
fn serde_name_named_struct() {
    test_case! {
        #[serde(rename= "Hello")]
        struct Test {
            s: String,
            num: Usize,
        }
    }

    assert!(Test::export()
        .to_zod_string()
        .starts_with("export const Hello"));

    assert!(Test::export()
        .to_ts_string()
        .starts_with("export interface Hello"));

    compare(
        Test::export().to_ts_string(),
        "export interface Hello { s: Rs.String, num: Rs.Usize }",
    )
}

#[test]
fn serde_name_tuple_str() {
    test_case! {
    #[serde(rename= "HelloTuple")]
    struct Test(String);
    }
    assert!(Test::export()
        .to_zod_string()
        .starts_with("export const HelloTuple"));
    compare(
        Test::export().to_ts_string(),
        "export type HelloTuple = Rs.String;",
    );
}

#[test]
fn serde_rename_struct_field() {
    test_case! {
        struct Test {
            #[serde(rename= "after")]
            before: String,
            other: Usize,
        }
    }

    assert!(Test::export().to_zod_string().contains("after"));
    assert!(Test::export().to_zod_string().contains("other"));
    assert!(!Test::export().to_zod_string().contains("before"));
}
