use std::collections::HashMap;

use crate::{rpc::codegen, rpc::Request, rpc::ResponseSender};

pub type StreamHandle = tokio::task::JoinHandle<()>;

#[derive(Debug, Default)]
pub struct SubscriberMap {
    inner: HashMap<usize, StreamHandle>,
}

impl std::ops::Deref for SubscriberMap {
    type Target = HashMap<usize, StreamHandle>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for SubscriberMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Drop for SubscriberMap {
    fn drop(&mut self) {
        for (_, jh) in self.inner.drain() {
            jh.abort();
        }
    }
}

#[async_trait::async_trait]
pub trait Backend {
    const NS_NAMES: &'static [&'static str];

    fn generate<T>() -> String
    where
        T: codegen::ClientCodegen;

    async fn handle_request(
        &mut self,
        req: Request,
        res: ResponseSender,
        subscribers: &mut SubscriberMap,
    );
}
