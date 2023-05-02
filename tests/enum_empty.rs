mod test_utils;
use test_utils::*;

#[test]
fn serde_empty_enum() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {}
    }

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([]));"#,
        "export type Test = never;",
    );
}
