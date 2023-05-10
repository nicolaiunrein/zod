mod api;
use api::AppBackend;
use axum::{extract::Extension, routing::get, Router, Server};
use zod::{core::rpc::server::Backend, server::BackendProxy};
use zod_axum::websocket_handler;

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

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
    let content = AppBackend::generate();
    println!("{content}");
}

async fn serve() {
    let backend = AppBackend(api::Chat::default());
    let proxy = BackendProxy::new(backend);

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .layer(Extension(proxy));

    Server::bind(&"127.0.0.1:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
