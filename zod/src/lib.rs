pub use zod_core::*;
pub use zod_derive::*;

pub mod rpc {
    pub use zod_rpc::*;
}

#[doc(hidden)]
pub mod __private {
    pub use inventory;
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/*.rs");
}
