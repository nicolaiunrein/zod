use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
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
fn externally_tagged() {
    let json = serde_json::to_value(Test::B(123)).unwrap();
    assert_eq!(json, serde_json::json!({"B": 123}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.union([z.object({{A: {string_schema}}}), z.object({{B: {number_schema}}})])")
    );
    assert_eq!(Test::type_def(), "{ A: string } | { B: number }");
    assert_eq!(Test::type_name(), "Ns.Test");
}
