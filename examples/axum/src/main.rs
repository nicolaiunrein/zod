use futures::Stream;
use futures::StreamExt;
use remotely::clients::WebsocketClient;
use remotely::servers::AxumWsServer;
use remotely::Backend;
use remotely::Server;
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
        Some("serve") => serve().await,
        _ => eprintln!("Call with serve or generate"),
    }
}

async fn serve() {
    let backend = MyBackend(Watchout { shared_data: 0 });
    let server = AxumWsServer::new(3000);
    server.serve(backend).await.unwrap();
}

fn generate() {
    let files = MyBackend::generate::<WebsocketClient>();
    for (name, content) in files.iter() {
        let name = name.display();
        println!("// {name}\n{content}\n\n")
    }
}
