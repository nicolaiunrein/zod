use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
enum Test {
    A { s: String, num: usize },
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
    let json = serde_json::to_value(Test::A {
        s: String::new(),
        num: 123,
    })
    .unwrap();
    assert_eq!(json, serde_json::json!({"A": {"s": "", "num": 123}}));

    let string_schema = String::schema();
    let number_schema = usize::schema();
    assert_eq!(
        Test::schema(),
        format!("z.object({{A: z.object({{ s: {string_schema}, num: {number_schema} }})}})")
    );
    assert_eq!(Test::type_def(), "{ A: { s: string, num: number } }");
    assert_eq!(Test::type_name(), "Ns.Test");
}
