mod test_utils;
use test_utils::*;

#[test]
fn serde_name_struct() {
    test_case! {
        #[serde(rename= "Hello")]
        enum Test {
            HelloWorld { s: String },
            AnotherValue { num: Usize },
        }
    }

    compare_export::<Test>(
        "export const Hello = z.lazy(() => z.union([z.object({ HelloWorld: z.object({ s: Rs.String }) }), z.object({ AnotherValue: z.object({ num: Rs.Usize }) })]));",
        "export type Hello = { HelloWorld: { s: Rs.String }} | { AnotherValue: { num: Rs.Usize }};",
    );
}

#[test]
fn serde_name_tuple() {
    test_case! {
        #[serde(rename= "Hello")]
        enum Test {
            HelloWorld(String, Usize),
            AnotherValue(Usize, Usize)
        }
    }
    compare_export::<Test>(
        "export const Hello = z.lazy(() => z.union([z.object({ HelloWorld: z.tuple([Rs.String, Rs.Usize]) }), z.object({ AnotherValue: z.tuple([Rs.Usize, Rs.Usize]) })]));",
        "export type Hello = { HelloWorld: [Rs.String, Rs.Usize]} | { AnotherValue: [Rs.Usize, Rs.Usize]};",
    );
}

#[test]
fn serde_name_unit() {
    test_case! {
        #[serde(rename= "Hello")]
        enum Test {
            HelloWorld,
            AnotherValue
        }

    }

    compare_export::<Test>(
        r#"export const Hello = z.lazy(() => z.union([z.literal("HelloWorld"), z.literal("AnotherValue")]));"#,
        r#"export type Hello = "HelloWorld" | "AnotherValue";"#,
    );
}
