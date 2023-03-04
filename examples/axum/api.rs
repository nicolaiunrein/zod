use futures::Stream;
use futures::StreamExt;
use zod::{rpc, Namespace, Zod};

#[derive(serde::Serialize, serde::Deserialize, Zod, Debug)]
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

#[derive(serde::Serialize, serde::Deserialize, Zod, Debug)]
#[zod(namespace = "Pixera")]
pub struct MyEntity2 {
    value: usize,
}

#[derive(Namespace)]
pub struct Watchout {
    pub shared_data: usize,
}

#[derive(zod::Namespace)]
pub struct Pixera {
    pub shared_data: usize,
}

#[rpc::namespace]
impl Pixera {
    fn x(&mut self) -> impl Stream<Item = String> {
        futures::stream::once(async move { String::new() })
    }

    // fn iter(&mut self) -> impl std::iter::Iterator<Item = String> {
    // // futures::stream::once(async move { String::new() })
    // std::iter::once(String::new())
    // }

    fn y(&mut self) -> std::pin::Pin<Box<dyn Stream<Item = String> + Send>> {
        futures::stream::once(async move { String::new() }).boxed()
    }
}

#[rpc::namespace]
impl Watchout {
    pub async fn nested(&mut self, _value: MyEntity) -> usize {
        self.shared_data += 1;
        self.shared_data
    }

    pub async fn hello1(&mut self, _s: String) -> usize {
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

    // pub fn hello_fail(&mut self, num: usize) -> impl std::iter::Iterator<Item = usize> {
    //     std::iter::once(0)
    // }
}

#[derive(rpc::Backend)]
pub struct MyBackend(pub Watchout, pub Pixera);
