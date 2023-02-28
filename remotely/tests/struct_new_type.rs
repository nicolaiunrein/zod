use remotely::{test_case, zod};
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

fn main() {}

#[test]
fn ok() {
    test_case! {
        struct Test(usize);
    }

    let json = serde_json::to_string(&Test(123)).unwrap();
    assert_eq!(json, "123");

    assert_eq!(Test::schema(), usize::schema());
    assert_eq!(Test::type_def(), usize::type_def());
    assert_eq!(Test::type_name(), "Ns.Test")
}
