use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[serde(tag = "type")]
enum Test {
    A { s: String, num: usize },
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
fn internally_tagged() {
    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"type": "B", "num": 123}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.discriminatedUnion(\"type\", [z.object({{ type: z.literal(\"A\"), s: {string_schema}, num: {number_schema} }}), z.object({{ type: z.literal(\"B\"), num: {number_schema} }})])")
    );
    assert_eq!(
        Test::type_def(),
        "{ type: \"A\", s: string, num: number } | { type: \"B\", num: number }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}
