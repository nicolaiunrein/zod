mod api;
// use api::MyBackend;
use axum::{extract::Extension, routing::get, Router, Server};
use zod::{
    core::rpc::server::Backend,
    rpc::{
        clients::WebsocketClient,
        servers::{axum::websocket_handler, proxy::BackendProxy},
    },
    types::Usize,
};

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    // match std::env::args().nth(1).as_deref() {
    // Some("generate") => generate(),
    // Some("serve") => serve().await,
    // _ => eprintln!("Call with serve or generate"),
    // }
}

// fn generate() {
// let content = MyBackend::generate::<WebsocketClient>();
// println!("{content}");
// }

// async fn serve() {
// let backend = MyBackend(
// api::Watchout {
// shared_data: Usize(0),
// },
// api::Pixera {
// shared_data: Usize(0),
// },
// );
// let proxy = BackendProxy::new(backend);

// let app = Router::new()
// .route("/ws", get(websocket_handler))
// .layer(Extension(proxy));

// Server::bind(&"127.0.0.1:8000".parse().unwrap())
// .serve(app.into_make_service())
// .await
// .unwrap();
// }
