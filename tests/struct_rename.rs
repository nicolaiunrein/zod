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

    assert!(Test::CODE.schema.starts_with("export const Hello"));
    assert!(Test::CODE.type_def.starts_with("export interface Hello"));
    compare(
        Test::CODE.type_def,
        "export interface Hello { s: Rs.String, num: Rs.Usize,}",
    )
}

#[test]
fn serde_name_tuple_str() {
    test_case! {
    #[serde(rename= "HelloTuple")]
    struct Test(String);
    }
    assert!(Test::CODE.schema.starts_with("export const HelloTuple"));
    compare(Test::CODE.type_def, "export type HelloTuple = Rs.String;");
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

    assert!(Test::CODE.schema.contains("after"));
    assert!(Test::CODE.schema.contains("other"));
    assert!(!Test::CODE.schema.contains("before"));
}
