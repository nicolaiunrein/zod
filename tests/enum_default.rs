mod test_utils;
use test_utils::*;

#[test]
fn serde_default_newtype_value() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            A(#[serde(default)]String),
            B(u16),
        }
    }

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([
            z.object({ A: Rs.String.optional() }),
            z.object({ B: Rs.U16 })
        ]));"#,
        "export type Test = { A: Rs.String | undefined } | { B: Rs.U16 };",
    );
}

#[test]
fn serde_default_enum_struct_field() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            A{
                #[serde(default)]
                s: String
            },
            B(u16),
        }
    }

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([
            z.object({ A: z.object({ s: Rs.String.optional() }) }),
            z.object({ B: Rs.U16 })
        ]));"#,
        "export type Test = { A: { s?: Rs.String | undefined }} | { B: Rs.U16 };",
    );
}

#[test]
fn serde_default_enum_tuple_field() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            A(#[serde(default)] String, u16),
            B(u16),
        }
    }

    compare_export::<Test>(
        r#"export const Test = z.lazy(() => z.union([
            z.object({ A: z.tuple([Rs.String.optional(), Rs.U16]) }),
            z.object({ B: Rs.U16 })
        ]));"#,
        "export type Test = { A: [Rs.String | undefined, Rs.U16] } | { B: Rs.U16 };",
    );
}
