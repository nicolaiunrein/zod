use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
struct Test(usize);

struct Ns {}

impl Namespace for Ns {
    const NAME: &'static str = "Ns";
    type Req = NsReq;
}

#[derive(serde::Deserialize)]
struct NsReq;

fn main() {}

#[test]
fn ok() {
    let json = serde_json::to_string(&Test(123)).unwrap();
    assert_eq!(json, "123");
    assert_eq!(Test::schema(), usize::schema());
}
