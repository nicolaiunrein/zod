use remotely::test_case;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

#[test]
fn serde_transparent_struct() {
    test_case! {
        #[derive(serde::Deserialize)]
        #[serde(transparent)]
        struct Test {
            s: String,
        }
    }

    assert_eq!(Test::schema(), String::schema());
    assert_eq!(Test::type_def(), String::type_def());
    assert_eq!(Test::type_name(), "Ns.Test")
}

#[test]
fn serde_transparent_newtype_struct() {
    test_case! {
        #[derive(serde::Deserialize)]
        #[serde(transparent)]
        struct Test(String);
    }

    assert_eq!(Test::schema(), String::schema());
    assert_eq!(Test::type_def(), String::type_def());
    assert_eq!(Test::type_name(), "Ns.Test")
}