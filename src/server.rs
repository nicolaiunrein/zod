//! Definition and helpers to implement an RPC server

use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    SinkExt, Stream, StreamExt,
};
use pin_project_lite::pin_project;
use zod_core::rpc::{server::Backend, Request, Response};

#[async_trait::async_trait]
pub trait Server {
    async fn serve<T>(self, backend: T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Backend + Send,
        Self: Sized;
}

#[derive(Clone, Debug)]
pub struct BackendProxy {
    tx: UnboundedSender<(Result<Request, Response>, UnboundedSender<Response>)>,
}

impl BackendProxy {
    pub fn new<T>(mut backend: T) -> Self
    where
        T: Backend + Send + 'static,
    {
        let (tx, mut rx) = unbounded();
        let mut subscribers = Default::default();

        tokio::spawn(async move {
            while let Some((req, mut res)) = rx.next().await {
                tracing::debug!(?req, "Incoming Request");
                match req {
                    Ok(req) => backend.forward_request(req, res, &mut subscribers).await,
                    Err(err) => {
                        if let Err(err) = res.send(err).await {
                            tracing::warn!(?err);
                        }
                    }
                }
            }
        });

        Self { tx }
    }

    pub fn connect(&self) -> ProxyConnection {
        let (res_tx, res_rx) = unbounded();
        ProxyConnection {
            tx: self.tx.clone(),
            res_tx,
            res_rx,
        }
    }
}

pub struct ProxyConnection {
    tx: UnboundedSender<(Result<Request, Response>, UnboundedSender<Response>)>,
    res_tx: UnboundedSender<Response>,
    res_rx: UnboundedReceiver<Response>,
}

impl ProxyConnection {
    pub fn split(self) -> (ProxyTx, ProxyRx) {
        let ProxyConnection { tx, res_tx, res_rx } = self;
        (ProxyTx { tx, res_tx }, ProxyRx { res_rx })
    }
}

pub struct ProxyTx {
    tx: UnboundedSender<(Result<Request, Response>, UnboundedSender<Response>)>,
    res_tx: UnboundedSender<Response>,
}

impl ProxyTx {
    pub fn send(&self, req: Result<Request, Response>) -> Result<(), ClientError> {
        self.tx
            .unbounded_send((req, self.res_tx.clone()))
            .map_err(|_| ClientError::Disconnected)
    }
}

pin_project! {
    pub struct ProxyRx {
        #[pin]
        res_rx: UnboundedReceiver<Response>,
    }
}

impl Stream for ProxyRx {
    type Item = Response;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        this.res_rx.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.res_rx.size_hint()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Client disconnected")]
    Disconnected,
}
