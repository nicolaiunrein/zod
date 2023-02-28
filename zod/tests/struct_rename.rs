use pretty_assertions::assert_eq;
use zod::{Codegen, Namespace};

mod test_utils;

#[test]
fn serde_name_named_struct() {
    test_case! {
        #[serde(rename= "Hello")]
        struct Test {
            s: String,
            num: usize,
        }
    }

    assert_eq!(Test::type_name(), "Ns.Hello");
}

#[test]
fn serde_name_tuple_str() {
    test_case! {
        #[serde(rename= "Hello")]
        struct Test(String);
    }
    assert_eq!(Test::type_name(), "Ns.Hello");
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

    assert!(Test::schema().contains("after"));
    assert!(Test::schema().contains("other"));
    assert!(!Test::schema().contains("before"));
}
