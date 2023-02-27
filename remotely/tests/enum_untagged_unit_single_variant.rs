use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[serde(untagged)]
enum Test {
    A,
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
    let json = serde_json::to_value(Test::A).unwrap();
    assert_eq!(json, serde_json::json!(null));

    assert_eq!(Test::schema(), String::from("z.null()"));
    assert_eq!(Test::type_def(), "null");
    assert_eq!(Test::type_name(), "Ns.Test");
}
