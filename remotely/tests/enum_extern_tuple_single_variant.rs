use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
enum Test {
    A(usize),
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
    let json = serde_json::to_value(Test::A(123)).unwrap();
    assert_eq!(json, serde_json::json!({"A": 123}));

    let number_schema = usize::schema();
    assert_eq!(Test::schema(), format!("z.object({{A: {number_schema}}})"));
    assert_eq!(Test::type_def(), "{ A: number }");
    assert_eq!(Test::type_name(), "Ns.Test");
}
