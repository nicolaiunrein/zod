use futures::{
    channel::mpsc::{unbounded, UnboundedSender},
    Stream, StreamExt,
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use zod::{
    rpc::{clients::WebsocketClient, rpc, Backend, Request, Response, SubscriberMap},
    Zod,
};

#[derive(serde::Serialize, serde::Deserialize, Zod)]
#[zod(namespace = "Watchout")]
pub struct MyEntity {
    value: MyEntity2,
}

#[derive(serde::Serialize, serde::Deserialize, Zod)]
#[zod(namespace = "Pixera")]
pub struct MyEntity2 {
    value: usize,
}

#[derive(zod::Namespace)]
pub struct Watchout {
    shared_data: usize,
}

#[derive(zod::Namespace)]
pub struct Pixera;

#[rpc]
impl Watchout {
    pub async fn hello(&mut self, _s: String, _n: usize) -> usize {
        self.shared_data += 1;
        self.shared_data
    }

    pub fn hello_stream(&mut self, num: usize) -> impl Stream<Item = usize> {
        futures::stream::iter(0..).take(num).then(|x| async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            x
        })
    }
}

#[rpc]
impl Pixera {}

#[derive(Backend)]
struct MyBackend(Watchout, Pixera);

struct Server {
    tx: UnboundedSender<Response>,
    backend: MyBackend,
    subscribers: SubscriberMap,
}

impl Server {
    async fn handle_request(&mut self, req: Request) {
        self.backend
            .handle_request(req, self.tx.clone(), &mut self.subscribers)
            .await;
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    match std::env::args().nth(1).as_deref() {
        Some("generate") => generate(),
        Some("method") => method().await,
        Some("stream") => {
            let (tx, mut rx) = unbounded();
            let backend = MyBackend(Watchout { shared_data: 0 }, Pixera);
            let mut server = Server {
                tx,
                backend,
                subscribers: Default::default(),
            };

            for id in 0..10 {
                stream(&mut server, id).await;
            }

            while let Some(Response::Stream { data, id }) = rx.next().await {
                if data == serde_json::json!(id) {
                    let json = serde_json::json!({"cancelStream": { "id": id}});
                    let req = serde_json::from_value(json).unwrap();
                    server.handle_request(req).await;
                }
                println!("{data:?}")
            }
        }
        _ => eprintln!("Call with method, stream or generate"),
    }
}

async fn method() {
    let (tx, mut rx) = unbounded();
    let backend = MyBackend(Watchout { shared_data: 0 }, Pixera);
    let mut server = Server {
        tx,
        backend,
        subscribers: Default::default(),
    };

    let json = serde_json::json!({"exec": {"id": 1, "namespace": "Watchout", "method": "hello", "args": ["abc", 123]}});
    let req = serde_json::from_value(json).unwrap();

    server.handle_request(req).await;

    let res = rx.next().await.unwrap();

    println!("{res:?}")
}

async fn stream(server: &mut Server, id: usize) {
    let json = serde_json::json!({"exec": {"id": id, "namespace": "Watchout", "method": "hello_stream", "args": [123]}});

    let req = serde_json::from_value(json).unwrap();
    server.handle_request(req).await;
}

fn generate() {
    let content = MyBackend::generate::<WebsocketClient>();
    println!("{content}");
}
