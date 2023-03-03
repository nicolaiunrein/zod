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

    assert!(Test::schema().contains("z.string().optional()"));
    assert_eq!(Test::type_def(), "{s?: string | undefined,\nnum: number}")
}

#[test]
fn serde_default_tuple_struct_field() {
    test_case! {
        #[derive(serde::Deserialize)]
        struct Test(#[serde(default)] String);
    }

    assert!(Test::schema().contains("z.string().optional()"));
    assert_eq!(Test::type_def(), "string | undefined")
}

#[test]
fn flatten() {
    test_case! {
        #[derive(Debug, serde::Deserialize, PartialEq)]
        struct Test(#[serde(default)]usize);
    }

    assert_eq!(Test::schema(), optional(usize::schema()));
    assert_eq!(Test::type_def(), "number | undefined");
    assert_eq!(Test::inline().to_string(), "Ns.Test")
}
