use remotely::test_case;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

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
    assert_eq!(Test::type_def(), "{s?: string | undefined,\nnum: number,}")
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
