use crate::remotely_core::{Backend, ClientCodegen};

#[async_trait::async_trait]
pub trait Server {
    async fn serve<T>(self, backend: T)
    where
        T: Backend + Send,
        Self: Sized;
}

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
        let req =
            serde_json::json!({"namespace": "Watchout", "method": "hello", "args": ["abc", 123]});

        let res = backend.handle_request(req).await;

        println!("{res:?}")
    }
}

pub struct WebsocketClient;

impl ClientCodegen for WebsocketClient {
    fn get() -> String {
        String::from(include_str!("./clients/websocket_client.ts"))
    }
}
