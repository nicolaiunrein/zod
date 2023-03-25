use pretty_assertions::assert_eq;

mod test_utils;
use test_utils::*;

#[test]
fn serde_skip_struct_field() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Test {
            #[serde(skip)]
            to_be_skipped: String,
            num: Usize,
        }
    }

    let value = Test {
        to_be_skipped: String::new(),
        num: Usize(123),
    };

    assert_eq!(
        value,
        serde_json::from_value(serde_json::json!({"num": "123"})).unwrap()
    );
    assert!(!Test::export()
        .unwrap()
        .to_zod_string()
        .contains("to_be_skipped"));
}

// #[test]
// fn serde_skip_deserializing_struct_field() {
//     test_case! {
//         #[derive(Debug, PartialEq, serde::Deserialize)]
//         struct Test {
//             #[serde(skip_deserializing)]
//             to_be_skipped: String,
//             num: Usize,
//         }
//     }
//
//     let value = Test {
//         to_be_skipped: String::new(),
//         num: Usize(123),
//     };
//
//     assert_eq!(
//         value,
//         serde_json::from_value(serde_json::json!({"num": "123"})).unwrap()
//     );
//
//     assert!(!Test::export()
//         .unwrap()
//         .to_zod_string()
//         .contains("to_be_skipped"));
// }
