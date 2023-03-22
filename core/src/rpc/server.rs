use std::collections::{BTreeMap, HashMap};

use crate::{ast::rpc, rpc::Request, rpc::ResponseSender, Namespace, Register};

use crate::types::Rs;

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
pub trait Backend: Register {
    fn generate<T>() -> String
    where
        T: rpc::ClientCodegen,
        Self: 'static,
    {
        let mut out = T::get();
        let mut exports = BTreeMap::<&str, Vec<_>>::new();
        for export in Self::dependencies().resolve().into_iter() {
            exports.entry(export.ns()).or_default().push(export);
        }

        if let Some(rs) = exports.remove(Rs::NAME) {
            out.push_str("export namepace ");
            out.push_str(Rs::NAME);
            out.push_str(" {\n");
            for node in rs.into_iter() {
                out.push_str(&node.to_string());
            }
            out.push_str("\n}\n")
        }

        for (ns, nodes) in exports.into_iter() {
            out.push_str("export namepace ");
            out.push_str(ns);
            out.push_str(" {\n");
            for node in nodes.into_iter() {
                out.push_str(&node.to_string());
            }
            out.push_str("\n}\n")
        }

        out
    }

    async fn handle_request(
        &mut self,
        req: Request,
        res: ResponseSender,
        subscribers: &mut SubscriberMap,
    );
}
