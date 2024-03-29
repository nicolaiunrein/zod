mod test_utils;
use zod::ZodType;

#[test]
fn serde_skip_enum_tuple_variant() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            A(#[serde(default)]String),
            B(usize),
        }
    }

    assert!(Test::schema().contains("z.string().optional()"));
    assert_eq!(
        Test::type_def(),
        "{ A: string | undefined } | { B: number }"
    );
}

//
#[test]
fn serde_skip_enum_struct_variant() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            A{
                #[serde(default)]
                s: String
            },
            B(usize),
        }
    }

    assert!(Test::schema().contains("z.string().optional()"));
    assert_eq!(
        Test::type_def(),
        "{ A: { s?: string | undefined } } | { B: number }"
    )
}
//
#[test]
fn serde_skip_enum_tuple_field() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        enum Test {
            A(#[serde(default)] String, usize),
            B(usize),
        }
    }

    assert!(Test::schema().contains("z.string().optional()"));
}
