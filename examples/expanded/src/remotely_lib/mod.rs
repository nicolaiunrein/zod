pub mod clients;
pub mod servers;

use crate::remotely_core::Backend;

#[async_trait::async_trait]
pub trait Server {
    async fn serve<T>(self, backend: T)
    where
        T: Backend + Send,
        Self: Sized;
}
