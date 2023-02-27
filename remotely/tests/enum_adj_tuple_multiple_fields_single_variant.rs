use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[serde(tag = "type", content = "content")]
enum Test {
    A(usize, usize),
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
    let json = serde_json::to_value(Test::A(123, 42)).unwrap();
    assert_eq!(json, serde_json::json!({"type": "A", "content": [123, 42]}));

    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.object({{ type: z.literal(\"A\"), content: z.tuple([{number_schema}, {number_schema}])}})")
    );
    assert_eq!(
        Test::type_def(),
        "{ type: \"A\", content: [number, number] }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}
