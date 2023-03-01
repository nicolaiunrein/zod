use futures::{Stream, StreamExt};
use zod::Zod;

mod generated;

#[derive(serde::Serialize, serde::Deserialize, Zod)]
#[zod(namespace = "Watchout")]
pub struct MyEntity {
    value: MyEntity2,
}

#[derive(serde::Serialize, serde::Deserialize, Zod)]
#[zod(namespace = "Watchout")]
pub struct T(usize);

mod nested_mod {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, Zod)]
    #[zod(namespace = "Watchout")]
    pub struct MyEntity3 {
        value: MyEntity2,
    }
}

#[derive(serde::Serialize, serde::Deserialize, Zod)]
#[zod(namespace = "Pixera")]
pub struct MyEntity2 {
    value: usize,
}

#[derive(zod::Namespace)]
#[namespace(name = "abc")]
pub struct Watchout {
    pub shared_data: usize,
}

#[derive(zod::Namespace)]
pub struct Pixera {
    pub shared_data: usize,
}

impl Watchout {
    pub async fn _nested(&mut self, _value: MyEntity) -> usize {
        self.shared_data += 1;
        self.shared_data
    }

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

pub struct MyBackend(pub Watchout, pub Pixera);
