use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[serde(tag = "type")]
enum Test {
    A,
    B,
}

struct Ns {}

impl Namespace for Ns {
    const NAME: &'static str = "Ns";
    type Req = NsReq;
}

#[derive(serde::Deserialize)]
struct NsReq;

fn main() {}

#[test]
fn internally_tagged() {
    let json = serde_json::to_value(Test::B).unwrap();
    assert_eq!(json, serde_json::json!({"type": "B"}));

    assert_eq!(
        Test::schema(),
        format!(
            "z.discriminatedUnion(\"type\", [z.object({{ type: z.literal(\"A\") }}), z.object({{ type: z.literal(\"B\") }})])"
        )
    );
    assert_eq!(Test::type_def(), "{ type: \"A\" } | { type: \"B\" }");
    assert_eq!(Test::type_name(), "Ns.Test");
}
