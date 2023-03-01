use std::collections::HashMap;
use std::fmt::Write;

use crate::{
    codegen::{self, RpcMember},
    Request, ResponseSender,
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
    fn is_member_of_self(_member: &'static RpcMember) -> bool;

    fn rpc_members() -> HashMap<&'static str, Vec<&'static RpcMember>> {
        let mut out = HashMap::<&'static str, Vec<&'static RpcMember>>::new();
        let members =
            inventory::iter::<RpcMember>().filter(|member| Self::is_member_of_self(member));

        for member in members {
            out.entry(member.ns_name()).or_default().push(member);
        }
        out
    }

    fn generate<T>() -> String
    where
        T: codegen::ClientCodegen,
    {
        let mut code = T::get();
        for (ns, members) in Self::rpc_members().into_iter() {
            let s: String = members.into_iter().map(|member| member.decl()).collect();
            write!(code, "export namespace {} {{ {} }}", ns, &s).expect("write failed");
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
