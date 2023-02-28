pub mod clients;
pub mod servers;

pub use zod_core::*;
pub use zod_rpc_core::error::Error;
pub use zod_rpc_core::server::Backend;
pub use zod_rpc_core::server::SubscriberMap;
pub use zod_rpc_core::Request;
pub use zod_rpc_core::Response;

#[async_trait::async_trait]
pub trait Server {
    async fn serve<T>(self, backend: T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Backend + Send,
        Self: Sized;
}

pub mod __private {
    pub use zod_rpc_core::*;
}
