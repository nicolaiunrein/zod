use std::marker::PhantomData;

use futures::Stream;
use futures::StreamExt;
use zod::{core::ast::Node, core::types::Usize, rpc, Namespace, Node};

#[derive(serde::Serialize, serde::Deserialize, Node, Debug)]
#[zod(namespace = "Watchout")]
pub struct MyEntity {
    value: String,
}

#[derive(serde::Serialize, serde::Deserialize, Node, Debug)]
#[zod(namespace = "Watchout")]
pub struct Generic<'a, T: Node, V: Node> {
    value: String,
    t: T,
    v: V,

    #[serde(skip)]
    _p: PhantomData<&'a T>,
}

#[derive(serde::Serialize, serde::Deserialize, Node, Debug)]
#[zod(namespace = "Watchout")]
pub struct User<'a> {
    value: Generic<'a, Usize, Usize>,
}

#[derive(serde::Serialize, serde::Deserialize, Node)]
#[zod(namespace = "Watchout")]
pub struct T(Usize);

mod nested_mod {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, Node)]
    #[zod(namespace = "Watchout")]
    pub struct MyEntity3 {
        value: MyEntity2,
    }
}

#[derive(serde::Serialize, serde::Deserialize, Node, Debug)]
#[zod(namespace = "Pixera")]
pub struct MyEntity2 {
    value: Usize,
}

#[derive(Namespace)]
pub struct Watchout {
    pub shared_data: Usize,
}

#[derive(zod::Namespace)]
pub struct Pixera {
    pub shared_data: Usize,
}

#[zod::rpc]
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

#[zod::rpc]
impl Watchout {
    pub async fn nested(&mut self, _value: MyEntity) -> Usize {
        *self.shared_data += 1;
        self.shared_data
    }

    pub async fn hello1(&mut self, _s: String) -> Usize {
        *self.shared_data += 1;
        self.shared_data
    }

    pub async fn hello(&mut self, _s: String, _n: Usize) -> Usize {
        *self.shared_data += 1;
        self.shared_data
    }

    pub async fn hello_user(&mut self, _user: User<'static>, _n: Usize) -> Usize {
        *self.shared_data += 1;
        self.shared_data
    }

    pub fn hello_stream(&mut self, num: Usize) -> impl Stream<Item = Usize> {
        futures::stream::iter(0..).take(*num).then(|x| async move {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            x.into()
        })
    }

    // pub fn hello_fail(&mut self, num: Usize) -> impl std::iter::Iterator<Item = Usize> {
    //     std::iter::once(0)
    // }
}

#[derive(zod::Backend)]
pub struct MyBackend(pub Watchout, pub Pixera);
