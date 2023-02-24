use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
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
fn externally_tagged() {
    let json = serde_json::to_value(Test::A).unwrap();
    assert_eq!(json, serde_json::json!("A"));

    assert_eq!(Test::schema(), format!("z.literal(\"A\")"));
    assert_eq!(Test::type_def(), "\"A\"");
    assert_eq!(Test::type_name(), "Ns.Test");
}