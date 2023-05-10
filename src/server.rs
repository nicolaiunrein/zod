//! Definition and helpers to implement an RPC server

use std::sync::atomic;

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
    event_tx: UnboundedSender<Event>,
}

pub enum Event {
    Disconnect {
        connection_id: usize,
    },
    Request {
        connection_id: usize,
        request: Result<Request, Response>,
        response_channel: UnboundedSender<Response>,
    },
}

impl BackendProxy {
    pub fn new<T>(mut backend: T) -> Self
    where
        T: Backend + Send + 'static,
    {
        let (tx, mut rx) = unbounded();
        let mut subscribers = Default::default();

        tokio::spawn(async move {
            while let Some(event) = rx.next().await {
                match event {
                    Event::Request {
                        connection_id,
                        request,
                        mut response_channel,
                    } => {
                        tracing::debug!(?request, "Incoming Request");
                        match request {
                            Ok(req) => {
                                backend
                                    .forward_request(
                                        connection_id,
                                        req,
                                        response_channel,
                                        &mut subscribers,
                                    )
                                    .await
                            }
                            Err(err) => {
                                if let Err(err) = response_channel.send(err).await {
                                    tracing::warn!(?err);
                                }
                            }
                        }
                    }
                    Event::Disconnect { connection_id } => {
                        println!("disconnecting {connection_id}");
                        subscribers.retain(|k, _| k.0 != connection_id);
                        println!("active = {}", subscribers.len())
                    }
                }
            }
        });

        Self { event_tx: tx }
    }

    pub fn connect(&self) -> ProxyConnection {
        static NEXT_ID: atomic::AtomicUsize = atomic::AtomicUsize::new(0);
        let connection_id = NEXT_ID.fetch_add(1, atomic::Ordering::SeqCst);
        let (res_tx, res_rx) = unbounded();
        ProxyConnection::new(ProxyConnectionInner {
            connection_id,
            tx: self.event_tx.clone(),
            res_tx,
            res_rx,
        })
    }
}

pub struct ProxyConnectionInner {
    connection_id: usize,
    tx: UnboundedSender<Event>,
    res_tx: UnboundedSender<Response>,
    res_rx: UnboundedReceiver<Response>,
}

pub struct ProxyConnection {
    inner: Option<ProxyConnectionInner>,
}

impl ProxyConnection {
    pub fn new(inner: ProxyConnectionInner) -> Self {
        Self { inner: Some(inner) }
    }
}

impl Drop for ProxyConnection {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.take() {
            let _ = inner.tx.unbounded_send(Event::Disconnect {
                connection_id: inner.connection_id,
            });
        }
    }
}

impl ProxyConnection {
    pub fn split(mut self) -> (ProxyTx, ProxyRx) {
        let ProxyConnectionInner {
            connection_id,
            tx,
            res_tx,
            res_rx,
        } = self.inner.take().unwrap();

        (
            ProxyTx {
                connection_id,
                tx,
                res_tx,
            },
            ProxyRx { res_rx },
        )
    }
}

pub struct ProxyTx {
    connection_id: usize,
    tx: UnboundedSender<Event>,
    res_tx: UnboundedSender<Response>,
}

impl Drop for ProxyTx {
    fn drop(&mut self) {
        let _ = self.tx.unbounded_send(Event::Disconnect {
            connection_id: self.connection_id,
        });
    }
}

impl ProxyTx {
    pub fn send(&self, request: Result<Request, Response>) -> Result<(), ClientError> {
        self.tx
            .unbounded_send(Event::Request {
                connection_id: self.connection_id,
                request,
                response_channel: self.res_tx.clone(),
            })
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
