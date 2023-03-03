use axum::{
    body::{boxed, Body, BoxBody},
    extract::ws::{Message, WebSocket},
    http,
    http::{HeaderValue, Response},
    response::IntoResponse,
    Extension,
};
use futures::{FutureExt, SinkExt, StreamExt};

use super::proxy::{BackendProxy, ProxyConnection};

pub struct RpcResponse(zod_core::rpc::Response);

impl IntoResponse for RpcResponse {
    fn into_response(self) -> Response<BoxBody> {
        let body: Body = serde_json::to_string(&self.0)
            .expect("failed to serialize body")
            .into();
        let mut resp = Response::new(boxed(body));
        resp.headers_mut().insert(
            http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        resp
    }
}

pub async fn websocket_handler(
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
                let req = serde_json::from_str(&json).map_err(|err| crate::rpc::Response::Error {
                    id: None,
                    data: crate::rpc::Error::from(err),
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
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    if let Err(err) = sender.send(Message::Text(json)).await {
                        tracing::warn!(?err, "failed to send response");
                        break;
                    }
                }
                Err(err) => {
                    panic!("failed to serialize response: {err}");
                }
            }
        }
    };

    futures::select! {
        _ = fut1.fuse() => {}
        _ = fut2.fuse() => {}
    }
}
