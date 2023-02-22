pub mod clients;
pub mod servers;

pub use remotely_core::Backend;

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
