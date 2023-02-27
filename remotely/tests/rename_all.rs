use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

macro_rules! test_case {
    ($($decl: tt)+) => {
        #[derive(zod, serde::Serialize)]
        #[zod(namespace = "Ns")]
        #[allow(dead_code)]
        $($decl)+

        struct Ns {}

        impl Namespace for Ns {
            const NAME: &'static str = "Ns";
            type Req = NsReq;
        }

        #[derive(serde::Deserialize)]
        struct NsReq;
    };
}

fn main() {}

#[test]
fn rename_all_struct() {
    test_case! {
        #[serde(rename_all = "camelCase")]
        enum Test {
            A { s: String },
            B { num: usize },
        }
    }

    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"b": {"num": 123}}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.union([z.object({{ a: z.object({{ s: {string_schema} }}) }}), z.object({{ b: z.object({{ num: {number_schema} }}) }})])")
    );
    assert_eq!(
        Test::type_def(),
        "{ a: { s: string } } | { b: { num: number } }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}
