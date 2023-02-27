use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
#[serde(untagged)]
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
fn externally_tagged() {
    let json = serde_json::to_value(Test::B { num: 123 }).unwrap();
    assert_eq!(json, serde_json::json!({"num": 123}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.union([z.object({{ s: {string_schema}, num: {number_schema} }}), z.object({{ num: {number_schema} }})])")
    );
    assert_eq!(
        Test::type_def(),
        "{ s: string, num: number } | { num: number }"
    );
    assert_eq!(Test::type_name(), "Ns.Test");
}
