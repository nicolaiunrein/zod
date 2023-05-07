use std::marker::PhantomData;

use futures::Stream;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use zod::{core::types::Usize, Namespace, RequestType, ResponseType};

#[derive(Namespace)]
pub struct Watchout {
    pub shared_data: Usize,
}

#[derive(zod::Namespace)]
pub struct Pixera {
    pub shared_data: Usize,
}

#[derive(Serialize, Deserialize, RequestType, ResponseType, Debug)]
#[zod(namespace = "Watchout")]
pub struct MyEntity {
    value: String,
}

#[derive(Serialize, Deserialize, RequestType, ResponseType, Debug, Clone)]
#[zod(namespace = "Watchout")]
#[serde(from = "MyEntity", into = "String")]
pub struct Newtype {
    value: String,
}

impl From<Newtype> for String {
    fn from(value: Newtype) -> Self {
        value.value
    }
}

impl From<MyEntity> for Newtype {
    fn from(value: MyEntity) -> Self {
        Self { value: value.value }
    }
}

#[derive(serde::Serialize, serde::Deserialize, RequestType, Debug)]
#[zod(namespace = "Watchout")]
pub struct Generic<'a, T: RequestType, V: RequestType> {
    value: String,
    t: T,
    v: V,

    #[serde(skip)]
    _p: PhantomData<&'a T>,
}

#[derive(serde::Serialize, serde::Deserialize, RequestType, Debug)]
#[zod(namespace = "Watchout")]
pub struct User<'a> {
    value: Generic<'a, Usize, Usize>,
}

#[derive(serde::Serialize, serde::Deserialize, RequestType)]
#[zod(namespace = "Watchout")]
pub struct T(Usize);

mod nested_mod {
    use super::*;

    #[derive(serde::Serialize, serde::Deserialize, RequestType)]
    #[zod(namespace = "Watchout")]
    pub struct MyEntity3 {
        value: MyEntity2,
    }
}

#[derive(serde::Serialize, serde::Deserialize, RequestType, Debug)]
#[zod(namespace = "Pixera")]
pub struct MyEntity2 {
    value: Usize,
}

#[zod::rpc]
impl Pixera {
    fn debug_stream(&mut self) -> impl Stream<Item = String> {
        futures::stream::once(async move { String::new() })
    }

    fn y(&mut self) -> std::pin::Pin<Box<dyn Stream<Item = String> + Send>> {
        futures::stream::once(async move { String::new() }).boxed()
    }

    pub fn hello_stream(&mut self, num: Usize) -> impl Stream<Item = Usize> {
        futures::stream::iter(0..).take(*num).then(|x| async move {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            Usize::from(x * 10000)
        })
    }
}

#[zod::rpc]
impl Watchout {
    pub async fn newtype(&mut self, value: Newtype) -> Newtype {
        value
    }

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

    pub async fn test(&mut self, _user: User<'static>, _n: Usize) -> Usize {
        *self.shared_data += 1;
        self.shared_data
    }
}

#[derive(zod::Backend)]
pub struct MyBackend(pub Watchout, pub Pixera);
