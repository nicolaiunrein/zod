use std::time::Duration;

use crate::Server;
use futures::{channel::mpsc::unbounded, StreamExt};
use remotely_core::Backend;

pub struct AxumWsServer {
    port: u16,
}

impl AxumWsServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

#[async_trait::async_trait]
impl Server for AxumWsServer {
    async fn serve<T>(self, mut backend: T)
    where
        T: Backend + Send,
        Self: Sized,
    {
        loop {
            let req = serde_json::json!({"namespace": "Watchout", "method": "hello", "args": ["abc", 123]});

            let stream_req =
                serde_json::json!({"namespace": "Watchout", "method": "hello_stream", "args": [4]});

            let (tx, mut rx) = unbounded();

            backend.handle_request(req, tx.clone()).await;
            backend.handle_request(stream_req, tx).await;

            while let Some(res) = rx.next().await {
                println!("{res:?}");
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
