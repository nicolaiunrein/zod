use zod::{Zod, ZodType};

mod test_utils;

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
        serde_json::json!({"true_value": true, "false_value": false})
    );
    assert_eq!(
        Test::schema(),
        format!("{}.extend({})", Base::schema(), Nested::schema())
    );
    assert_eq!(
        Test::type_def(),
        format!("{} & Ns.Nested", Base::type_def())
    );
    assert_eq!(Test::type_name(), "Ns.Test")
}
