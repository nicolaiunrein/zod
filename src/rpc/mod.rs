#![deny(unsafe_code)]

pub mod clients;
pub mod servers;

pub use zod_core::rpc::error::Error;
pub use zod_core::rpc::server::Backend;
pub use zod_core::rpc::server::SubscriberMap;
pub use zod_core::rpc::Request;
pub use zod_core::rpc::Response;

pub use zod_derive::namespace;
pub use zod_derive::Backend;

#[async_trait::async_trait]
pub trait Server {
    async fn serve<T>(self, backend: T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Backend + Send,
        Self: Sized;
}

#[doc(hidden)]
pub mod __private {
    pub use async_trait;
    pub use futures;
    pub use inventory;
    pub use serde;
    pub use serde_json;
    pub use tokio;
    pub use tracing;
    pub use zod_core::rpc::*;
}
