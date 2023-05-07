use pretty_assertions::assert_eq;

mod test_utils;
use test_utils::*;
use zod::ResponseType;

#[test]
fn serde_skip_struct_field() {
    test_case! {
        #[derive(Debug, PartialEq, serde::Deserialize, zod::ResponseType)]
        struct Test {
            #[serde(skip)]
            skip_both: String,

            #[serde(skip_deserializing)]
            skip_req: String,

            #[serde(skip_serializing)]
            skip_res: String,

            num: Usize,
        }
    }

    let value = Test {
        skip_both: String::new(),
        skip_req: String::new(),
        skip_res: String::new(),
        num: Usize(123),
    };

    assert_eq!(
        value,
        serde_json::from_value(serde_json::json!({"num": "123", "skip_res": ""})).unwrap()
    );

    let req_export = <Test as RequestType>::export().to_zod_string();
    let res_export = <Test as ResponseType>::export().to_zod_string();

    assert!(!req_export.contains("skip_both"));
    assert!(!req_export.contains("skip_req"));
    assert!(req_export.contains("skip_res"));

    assert!(!res_export.contains("skip_both"));
    assert!(!res_export.contains("skip_res"));
    assert!(res_export.contains("skip_req"));
}
