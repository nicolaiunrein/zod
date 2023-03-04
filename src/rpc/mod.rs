#![deny(unsafe_code)]

pub mod clients;
pub mod servers;

pub use zod_core::rpc::{error::Error, server::Backend, server::SubscriberMap, Request, Response};
pub use zod_derive::{namespace, Backend};

#[async_trait::async_trait]
pub trait Server {
    async fn serve<T>(self, backend: T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Backend + Send,
        Self: Sized;
}
