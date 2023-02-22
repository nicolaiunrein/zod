use axum::{
    extract::{
        connect_info::Connected,
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, TypedHeader,
    },
    headers,
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    StreamExt,
};
use futures::{FutureExt, SinkExt};
use hyper::server::conn::AddrStream;
use std::{
    net::SocketAddr,
    sync::atomic::{AtomicUsize, Ordering},
};

type ReqChannel = UnboundedSender<(serde_json::Value, UnboundedSender<serde_json::Value>)>;

#[derive(Clone, Debug)]
pub struct ClientInfo {
    pub id: usize,
    pub remote_addr: SocketAddr,
}

impl Connected<&AddrStream> for ClientInfo {
    fn connect_info(target: &AddrStream) -> Self {
        static ID: AtomicUsize = AtomicUsize::new(0);

        ClientInfo {
            remote_addr: target.remote_addr(),
            id: ID.fetch_add(1, Ordering::Relaxed),
        }
    }
}

async fn ws_handler(
    tx: ReqChannel,
    ConnectInfo(info): ConnectInfo<ClientInfo>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        tracing::info!(?info, "Client connected!");
        tracing::trace!(
            id = info.id,
            user_agen = user_agent.as_str(),
            "Connection Details"
        );
    }

    ws.on_upgrade(move |socket| handle_socket(tx, info, socket))
}

async fn handle_socket(tx: ReqChannel, client_info: ClientInfo, socket: WebSocket) {
    let (mut sender, receiver) = socket.split();

    let stream = receiver
        .take_while(|msg| {
            futures::future::ready(match msg {
                Ok(Message::Close(frame)) => {
                    let code = frame.as_ref().map(|frame| frame.code);
                    let reason = frame
                        .as_ref()
                        .map(|frame| frame.reason.clone())
                        .unwrap_or_default();

                    tracing::warn!(?code, ?reason, "Socket closed!",);
                    false
                }
                Err(err) => {
                    tracing::error!(%err, "Received Error");
                    false
                }
                _ => true,
            })
        })
        .filter_map(|msg| async move {
            match msg {
                Ok(Message::Text(text)) => match serde_json::from_str(&text) {
                    Ok(json) => Some(json),
                    Err(err) => {
                        tracing::warn!(?err, "Error deserializing");
                        None
                    }
                },
                Ok(Message::Binary(_)) => None,
                Ok(Message::Ping(_)) => None,
                Ok(Message::Pong(_)) => None,
                Ok(Message::Close(_)) => unreachable!(),
                Err(_) => unreachable!(),
            }
        });

    futures::pin_mut!(stream);

    let (cb_tx, mut cb_rx) = unbounded();

    tokio::spawn(async move {
        while let Some(out) = cb_rx.next().await {
            let txt = serde_json::to_string(&out).unwrap();
            sender.send(Message::Text(txt)).await.unwrap();
        }
    });

    while let Some(msg) = stream.next().await {
        tx.unbounded_send((msg, cb_tx.clone())).unwrap();
    }
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
impl crate::Server for AxumWsServer {
    async fn serve<T>(self, mut backend: T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: remotely_core::Backend + Send,
        Self: Sized,
    {
        let (tx, mut rx) = unbounded();
        let app = Router::new().route(
            "/ws",
            get(|info, ws, user_agent| ws_handler(tx, info, ws, user_agent)),
        );

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));

        tracing::info!("listening on {}", addr);

        let fut1 = axum::Server::bind(&addr)
            .serve(app.into_make_service_with_connect_info::<ClientInfo>());

        let fut2 = async {
            while let Some((json, out)) = rx.next().await {
                backend.handle_request(json, out).await;
            }
        };

        futures::select! {
            res = fut1.fuse() => {
                res?;
            },

            _ = fut2.fuse() => {},

        }

        Ok(())
    }
}
