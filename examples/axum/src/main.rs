use api::MyBackend;
use axum::extract::ws::Message;
use axum::extract::ws::WebSocket;
use axum::Server;
use futures::channel::mpsc::unbounded;
use futures::channel::mpsc::UnboundedReceiver;
use futures::channel::mpsc::UnboundedSender;
use futures::FutureExt;
use futures::SinkExt;
use futures::Stream;
use futures::StreamExt;
use pin_project_lite::pin_project;
use remotely::clients::WebsocketClient;
use remotely::Backend;
use remotely::Error;
use remotely::Request;
use remotely::Response;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use axum::{extract::Extension, response::IntoResponse, routing::get, Router};

mod api;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    match std::env::args().nth(1).as_deref() {
        Some("generate") => generate(),
        Some("serve") => serve().await,
        _ => eprintln!("Call with serve or generate"),
    }
}

fn generate() {
    let files = MyBackend::generate::<WebsocketClient>();
    for (name, content) in files.iter() {
        let name = name.display();
        println!("// {name}\n{content}\n\n")
    }
}

async fn serve() {
    let backend = MyBackend(api::Watchout { shared_data: 0 });
    let proxy = BackendProxy::new(backend);

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .layer(Extension(proxy));

    Server::bind(&"127.0.0.1:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

impl BackendProxy {
    pub fn new<T>(mut backend: T) -> Self
    where
        T: Backend + Send + 'static,
    {
        let (tx, mut rx) = unbounded();
        let mut subscribers = Default::default();

        tokio::spawn(async move {
            while let Some((result, mut res)) = rx.next().await {
                match result {
                    Ok(req) => backend.handle_request(req, res, &mut subscribers).await,
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

#[derive(Clone, Debug)]
struct BackendProxy {
    tx: UnboundedSender<(Result<Request, Response>, UnboundedSender<Response>)>,
}

struct ProxyConnection {
    tx: UnboundedSender<(Result<Request, Response>, UnboundedSender<Response>)>,
    res_tx: UnboundedSender<Response>,
    res_rx: UnboundedReceiver<Response>,
}

struct ProxyTx {
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

#[derive(thiserror::Error, Debug)]
enum ClientError {
    #[error("Client disconnected")]
    Disconnected,
}

pin_project! {
struct ProxyRx {
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

impl ProxyConnection {
    pub fn split(self) -> (ProxyTx, ProxyRx) {
        let ProxyConnection { tx, res_tx, res_rx } = self;
        (ProxyTx { tx, res_tx }, ProxyRx { res_rx })
    }
}

async fn websocket_handler(
    ws: axum::extract::WebSocketUpgrade,
    proxy: Extension<BackendProxy>,
) -> impl IntoResponse {
    let con = proxy.connect();
    ws.on_upgrade(|socket| websocket(socket, con))
}

async fn websocket(stream: WebSocket, con: ProxyConnection) {
    let (mut sender, mut receiver) = stream.split();
    let (tx, mut rx) = con.split();

    let fut1 = async move {
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Text(json) = message {
                let req = serde_json::from_str(&json).map_err(|err| Response::Error {
                    id: None,
                    data: Error::from(err),
                });
                if let Err(err) = tx.send(req) {
                    tracing::warn!(?err);
                    break;
                }
            }
        }
    };

    let fut2 = async move {
        while let Some(msg) = rx.next().await {
            let json = serde_json::to_string(&msg).unwrap();
            sender.send(Message::Text(json)).await.unwrap();
        }
    };

    futures::select! {
        _ = fut1.fuse() => {}
        _ = fut2.fuse() => {}
    }
}
