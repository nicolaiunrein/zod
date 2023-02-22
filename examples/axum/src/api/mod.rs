use futures::{Stream, StreamExt};

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
    pub shared_data: usize,
}

impl Watchout {
    pub async fn hello(&mut self, _s: String, _n: usize) -> usize {
        self.shared_data += 1;
        self.shared_data
    }

    pub fn hello_stream(&mut self, num: usize) -> impl Stream<Item = usize> {
        futures::stream::iter(0..).take(num).then(|x| async move {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            x
        })
    }
}

pub struct MyBackend(pub Watchout);
