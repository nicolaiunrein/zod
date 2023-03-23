#![deny(unsafe_code)]

pub mod clients;
pub mod servers;

pub use zod_core::rpc::{server::Backend, server::SubscriberMap, Error, Request, Response};

#[async_trait::async_trait]
pub trait Server {
    async fn serve<T>(self, backend: T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Backend + Send,
        Self: Sized;
}
