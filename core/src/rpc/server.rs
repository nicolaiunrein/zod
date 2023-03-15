use std::collections::HashMap;
use std::fmt::Write;

use crate::Code;
use crate::{
    rpc::codegen::{self, RpcMember},
    rpc::Request,
    rpc::ResponseSender,
};

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
    const DEPS: &'static [Code];
    const MEMBERS: &'static [RpcMember];

    fn member_map() -> HashMap<&'static str, Vec<String>> {
        let mut out = HashMap::<&'static str, Vec<String>>::new();

        for member in Self::MEMBERS {
            out.entry(member.ns_name()).or_default().push(member.decl());
        }

        for dep in Self::DEPS {
            let list = out.entry(dep.ns_name().unwrap_or_default()).or_default();
            list.push(dep.type_def.to_owned());
            list.push(dep.schema.to_owned());
        }
        out
    }

    fn generate<T>() -> String
    where
        T: codegen::ClientCodegen,
    {
        let mut code = T::get();

        for (ns, ns_code) in Self::member_map().into_iter() {
            if ns.is_empty() {
                write!(code, "{}", ns_code.join("\n")).expect("write failed");
            } else {
                write!(code, "export namespace {} {{ {} }}", ns, ns_code.join("\n"))
                    .expect("write failed");
            }
        }
        code
    }

    async fn handle_request(
        &mut self,
        req: Request,
        res: ResponseSender,
        subscribers: &mut SubscriberMap,
    );
}
