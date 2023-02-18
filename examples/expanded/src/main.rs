use futures::Stream;
use futures::StreamExt;
use remotely::clients::WebsocketClient;
use remotely::servers::AxumWsServer;
use remotely::Backend;
use remotely::Server;

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
    pub async fn hello(&mut self, s: String, n: usize) -> usize {
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
    let backend = MyBackend(Watchout { shared_data: 0 });

    match std::env::args().skip(1).next().as_ref().map(|s| s.as_str()) {
        Some("generate") => {
            let files = MyBackend::generate::<WebsocketClient>();
            for (name, content) in files.iter() {
                let name = name.display();
                println!("// {name}\n{content}\n\n")
            }
        }

        Some("serve") => {
            let server = AxumWsServer::new(3000);
            server.serve(backend).await;
        }

        _ => {
            eprintln!("Call with serve or generate")
        }
    }
}
