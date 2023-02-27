pub mod clients;
pub mod servers;

pub use remotely_core::error::Error;
pub use remotely_core::server::Backend;
pub use remotely_core::server::SubscriberMap;
pub use remotely_core::Request;
pub use remotely_core::Response;
pub use remotely_zod::*;

#[async_trait::async_trait]
pub trait Server {
    async fn serve<T>(self, backend: T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Backend + Send,
        Self: Sized;
}

pub mod __private {
    pub use remotely_core::*;
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/*.rs");
    // t.pass("tests/ui/pass/*.rs");
}

#[macro_export]
macro_rules! test_case {
    ($($decl: tt)+) => {
        #[derive(zod, serde::Serialize)]
        #[zod(namespace = "Ns")]
        #[allow(dead_code)]
        $($decl)+

        struct Ns {}

        impl Namespace for Ns {
            const NAME: &'static str = "Ns";
            type Req = NsReq;
        }

        #[derive(serde::Deserialize)]
        struct NsReq;
    };
}
