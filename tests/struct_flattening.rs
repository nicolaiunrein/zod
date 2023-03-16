use zod::{Zod, ZodType};

mod test_utils;
use pretty_assertions::assert_eq;
use test_utils::*;

#[test]
fn serde_flatten_struct() {
    test_case! {
        #[derive(serde::Deserialize)]
        struct Test {
            #[serde(flatten)]
            nested: Nested,
            false_value: bool
        }

        #[derive(serde::Deserialize, serde::Serialize, Zod)]
        #[zod(namespace = "Ns")]
        struct Nested {
            true_value: bool
        }

        #[derive(serde::Deserialize, serde::Serialize, Zod)]
        #[zod(namespace = "Ns")]
        struct Base{
            false_value: bool
        }

    }

    let s = serde_json::to_value(Test {
        nested: Nested { true_value: true },
        false_value: false,
    })
    .unwrap();

    assert_eq!(
        s,
        serde_json::json!({"true_value": true, "false_value": false}),
    );
    compare(
        Test::AST.schema,
        "export const Test = z.lazy(() => z.object({false_value: Rs.Bool})).extend(z.lazy(() => Ns.Nested));",
    );
    compare(
        Test::AST.type_def,
        "export interface Test extends Ns.Nested { false_value: Rs.Bool,}",
    );
}
