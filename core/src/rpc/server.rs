use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

use crate::NamespaceMemberDefinition;

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

#[derive(Clone, Debug, Default)]
pub struct CodegenOptions {
    prefix_schema: String,
    suffix_schema: String,

    prefix_type: String,
    suffix_type: String,

    prefix_interface: String,
    suffix_interface: String,
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
        Self::generate_with_options::<T>(Default::default())
    }

    fn generate_with_options<T>(options: CodegenOptions) -> String
    where
        T: codegen::ClientCodegen,
    {
        let rpc_records = inventory::iter::<RpcMember>()
            .filter(|member| Self::NS_NAMES.contains(&member.ns_name()))
            .map(|m| (m.ns_name(), m.decl()));

        let mut records: BTreeMap<&'static str, String> =
            crate::NamespaceMemberDefinition::collect()
                .into_iter()
                .filter(|(ns, _)| Self::NS_NAMES.contains(ns))
                .map(|(ns, defs)| {
                    (
                        ns,
                        defs.into_iter()
                            .map(|def| {
                                let td = match def.type_def() {
                                    crate::TsTypeDef::Interface(inner) => {
                                        format!(
                                            "export interface {}{}{} {}",
                                            options.prefix_interface,
                                            def.name(),
                                            options.suffix_interface,
                                            inner
                                        )
                                    }

                                    crate::TsTypeDef::Type(inner) => {
                                        format!(
                                            "export type {}{}{} = {};",
                                            options.prefix_type,
                                            def.name(),
                                            options.suffix_type,
                                            inner
                                        )
                                    }
                                };

                                format!(
                                    "export const {}{}{}= {}\n{}\n\n",
                                    options.prefix_schema,
                                    def.name(),
                                    options.suffix_schema,
                                    def.schema(),
                                    td
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
