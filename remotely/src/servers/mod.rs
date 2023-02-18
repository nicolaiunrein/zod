mod axum_server;
use futures::StreamExt;
use remotely_core::Backend;

pub use axum_server::AxumWsServer;

pub struct Server {
    jh: tokio::task::JoinHandle<()>,
    req_tx: futures::channel::mpsc::UnboundedSender<(
        serde_json::Value,
        futures::channel::mpsc::UnboundedSender<serde_json::Value>,
    )>,
}

impl Server {
    fn new<T>(mut backend: T) -> Self
    where
        T: Backend + Send + 'static,
    {
        let (req_tx, mut req_rx) = futures::channel::mpsc::unbounded::<(
            serde_json::Value,
            futures::channel::mpsc::UnboundedSender<serde_json::Value>,
        )>();

        let jh = tokio::spawn(async move {
            while let Some((req, cb)) = req_rx.next().await {
                backend.handle_request(req, cb);
            }
        });
        Self { jh, req_tx }
    }

    async fn req(&self, req: serde_json::Value) -> impl futures::Stream<Item = serde_json::Value> {
        let (res_tx, res_rx) = futures::channel::mpsc::unbounded();
        self.req_tx.unbounded_send((req, res_tx)).unwrap();

        res_rx
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.jh.abort();
    }
}
