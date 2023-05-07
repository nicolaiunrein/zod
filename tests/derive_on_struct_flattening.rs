mod test_utils;
use test_utils::*;

use pretty_assertions::assert_eq;

#[test]
fn serde_flatten_struct() {
    test_case! {
        #[derive(serde::Deserialize)]
        struct Test {
            #[serde(flatten)]
            nested: Nested,
            false_value: bool
        }

        #[derive(serde::Deserialize, serde::Serialize, RequestType)]
        #[zod(namespace = "Ns")]
        struct Nested {
            true_value: bool
        }

        #[derive(serde::Deserialize, serde::Serialize, RequestType)]
        #[zod(namespace = "Ns")]
        struct Base{
            false_value: bool
        }

    }

    let json = serde_json::to_value(Test {
        nested: Nested { true_value: true },
        false_value: false,
    })
    .unwrap();

    assert_eq!(
        json,
        serde_json::json!({"true_value": true, "false_value": false}),
    );

    compare_export::<Test>(
        "export const Test = z.lazy(() => z.object({false_value: Rs.Bool})).extend(z.lazy(() => Ns.Nested));",
        "export interface Test extends Ns.Nested { false_value: Rs.Bool }",
    );
}
