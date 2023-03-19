use std::collections::{BTreeMap, HashMap};

use crate::{rpc::codegen, rpc::Request, rpc::ResponseSender, DependencyRegistration, Namespace};

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
pub trait Backend: DependencyRegistration {
    fn generate<T>() -> String
    where
        T: codegen::ClientCodegen,
        Self: 'static,
    {
        let mut out = T::get();
        let mut exports = BTreeMap::<&str, Vec<_>>::new();
        for export in Self::dependencies().resolve().into_iter() {
            exports.entry(export.ns()).or_default().push(export);
        }

        if let Some(rs) = exports.remove(crate::build_ins::Rs::NAME) {
            out.push_str("export namepace ");
            out.push_str(crate::build_ins::Rs::NAME);
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
