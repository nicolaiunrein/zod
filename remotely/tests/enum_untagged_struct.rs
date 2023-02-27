use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[serde(untagged)]
#[allow(dead_code)]
enum Test {
    A { s: String },
    B { num: usize },
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
    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(
        json,
        serde_json::json!({"type": "B", "content": { "num": 123 }})
    );

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.discriminatedUnion(\"type\", [z.object({{ type: z.literal(\"A\"), content: z.object({{ s: {string_schema} }}) }}), z.object({{ type: z.literal(\"B\"), content: z.object({{ num: {number_schema} }}) }})])")
    );
    assert_eq!(
        Test::type_def(),
        "{ type: \"A\", content: { s: string } } | { type: \"B\", content: { num: number } }"
    );
    // assert_eq!(Test::type_name(), "Ns.Test");
}
