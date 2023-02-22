use futures::channel::mpsc::unbounded;
use futures::Stream;
use futures::StreamExt;
use remotely::__private::Request;
use remotely::clients::WebsocketClient;
use remotely::Backend;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

mod generated;

#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(rename = "Watchout.MyEntity")]
pub struct MyEntity {
    value: MyEntity2,
}

#[derive(serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(rename = "Pixera.MyEntity2")]
pub struct MyEntity2 {
    value: usize,
}

pub struct Watchout {
    shared_data: usize,
}

impl Watchout {
    pub async fn hello(&mut self, _s: String, _n: usize) -> usize {
        self.shared_data += 1;
        self.shared_data
    }

    pub fn hello_stream(&mut self, num: usize) -> impl Stream<Item = usize> {
        futures::stream::iter(0..).take(num)
    }
}

struct MyBackend(Watchout);

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    match std::env::args().nth(1).as_deref() {
        Some("generate") => generate(),
        Some("method") => method().await,
        Some("stream") => stream().await,
        _ => eprintln!("Call with method, stream or generate"),
    }
}

async fn method() {
    let mut backend = MyBackend(Watchout { shared_data: 0 });
    let req = Request::Method(
        serde_json::json!({"namespace": "Watchout", "method": "hello", "args": ["abc", 123]}),
    );

    let (tx, mut rx) = unbounded();
    backend.handle_request(req, tx).await;
    let res = rx.next().await.unwrap();
    println!("{res:?}")
}

async fn stream() {
    let mut backend = MyBackend(Watchout { shared_data: 0 });

    let req = Request::Method(
        serde_json::json!({"namespace": "Watchout", "method": "hello_stream", "arg": [123]}),
    );

    let (tx, mut rx) = unbounded();
    backend.handle_request(req, tx).await;
    let res = rx.next().await.unwrap();
    println!("{res:?}")
}

fn generate() {
    let files = MyBackend::generate::<WebsocketClient>();
    for (name, content) in files.iter() {
        let name = name.display();
        println!("// {name}\n{content}\n\n")
    }
}
