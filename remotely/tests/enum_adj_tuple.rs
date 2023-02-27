use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
#[serde(tag = "type", content = "content")]
enum Test {
    A(String),
    B(usize),
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
fn adj_tagged() {
    let json = serde_json::to_value(Test::B(123)).unwrap();
    assert_eq!(json, serde_json::json!({"type": "B", "content": 123}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.discriminatedUnion(\"type\", [z.object({{ type: z.literal(\"A\"), content: {string_schema}}}), z.object({{ type: z.literal(\"B\"), content: {number_schema}}})])")
    );
    assert_eq!(
        Test::type_def(),
        "{ type: \"A\", content: string } | { type: \"B\", content: number }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}
