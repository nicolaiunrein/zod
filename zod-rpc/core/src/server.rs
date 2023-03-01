use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

use zod_core::NamespaceMemberDefinition;

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
    const NS_NAMES: &'static [&'static str];

    fn rpc_members() -> HashMap<&'static str, Vec<String>> {
        let mut out = HashMap::<&'static str, Vec<String>>::new();
        let members = inventory::iter::<RpcMember>()
            .filter(|member| Self::NS_NAMES.contains(&member.ns_name()));

        for member in members {
            out.entry(member.ns_name()).or_default().push(member.decl());
        }
        out
    }

    fn zod_namespaces() -> HashMap<&'static str, Vec<&'static NamespaceMemberDefinition>> {
        let mut out = HashMap::<&'static str, Vec<&'static NamespaceMemberDefinition>>::new();
        let members = inventory::iter::<NamespaceMemberDefinition>()
            .filter(|member| Self::NS_NAMES.contains(&member.namespace()));

        for member in members {
            out.entry(member.namespace()).or_default().push(member);
        }
        out
    }

    fn generate<T>() -> String
    where
        T: codegen::ClientCodegen,
    {
        let rpc_records = inventory::iter::<RpcMember>()
            .filter(|member| Self::NS_NAMES.contains(&member.ns_name()))
            .map(|m| (m.ns_name(), m.decl()));

        let mut records: BTreeMap<&'static str, String> =
            zod_core::NamespaceMemberDefinition::collect()
                .into_iter()
                .filter(|(ns, _)| Self::NS_NAMES.contains(ns))
                .map(|(ns, defs)| {
                    (
                        ns,
                        defs.into_iter()
                            .map(|def| {
                                format!(
                                    "export const {}Schema = {}\nexport interface {} {}\n\n",
                                    def.name(),
                                    def.schema(),
                                    def.name(),
                                    def.type_def()
                                )
                            })
                            .collect(),
                    )
                })
                .collect();

        for (name, code) in rpc_records.into_iter() {
            let s = records.entry(name).or_default();
            write!(s, "{}", code).unwrap();
        }

        let mut code = T::get();

        for (ns, ns_code) in records.into_iter() {
            write!(code, "export namespace {} {{ {} }}", ns, ns_code).expect("write failed");
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
