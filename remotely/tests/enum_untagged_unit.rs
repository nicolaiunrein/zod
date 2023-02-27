use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
#[serde(untagged)]
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
fn externally_tagged() {
    let json = serde_json::to_value(Test::B).unwrap();
    assert_eq!(json, serde_json::json!(null));

    assert_eq!(Test::schema(), format!("z.union([z.null(), z.null()])"));
    assert_eq!(Test::type_def(), "null | null");
    assert_eq!(Test::type_name(), "Ns.Test");
}
