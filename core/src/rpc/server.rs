use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Display;

use crate::ast::rpc::RpcRequest;
use crate::{
    ast::rpc, ast::Export, rpc::Request, rpc::ResponseSender, Namespace, RequestTypeVisitor,
    ResponseTypeVisitor,
};

use crate::types::Rs;

/// a [JoinHandle](tokio::task::JoinHandle) to cancel a stream when it is dropped.
pub struct StreamHandle(tokio::task::JoinHandle<()>);

impl From<tokio::task::JoinHandle<()>> for StreamHandle {
    fn from(inner: tokio::task::JoinHandle<()>) -> Self {
        Self(inner)
    }
}

/// A map of active subscriber ids to the [JoinHandle](tokio::task::JoinHandle) for stream abortion
pub type SubscriberMap = HashMap<usize, StreamHandle>;

impl Drop for StreamHandle {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// This trait represents a collection of Namespaces
#[async_trait::async_trait]
pub trait Backend: RequestTypeVisitor + ResponseTypeVisitor {
    const AST: &'static [&'static [RpcRequest]];

    async fn forward_request(
        &mut self,
        req: Request,
        res: ResponseSender,
        subscribers: &mut SubscriberMap,
    );

    fn generate<T>() -> String
    where
        T: rpc::ClientCodegen,
        Self: 'static,
    {
        let mut out = T::get();
        let mut exports = BTreeMap::<&str, HashSet<ExportOrReq>>::new();

        #[derive(Hash, Eq, PartialEq)]
        enum ExportOrReq {
            Export(Export),
            Req(RpcRequest),
        }

        struct NamespaceExporter {
            name: &'static str,
            exports: Vec<ExportOrReq>,
        }

        impl NamespaceExporter {
            pub fn new(name: &'static str, mut exports: Vec<ExportOrReq>) -> Self {
                exports.sort_by(|a, b| match (a, b) {
                    (ExportOrReq::Export(a), ExportOrReq::Export(b)) => {
                        a.path.name().cmp(b.path.name())
                    }
                    (ExportOrReq::Req(a), ExportOrReq::Req(b)) => a.path.name().cmp(b.path.name()),
                    (ExportOrReq::Export(_), ExportOrReq::Req(_)) => std::cmp::Ordering::Less,
                    (ExportOrReq::Req(_), ExportOrReq::Export(_)) => std::cmp::Ordering::Greater,
                });
                Self { name, exports }
            }
        }

        impl Display for NamespaceExporter {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("export namespace ")?;
                f.write_str(self.name)?;
                f.write_str(" {\n")?;
                for export in self.exports.iter() {
                    match export {
                        ExportOrReq::Export(export) => export.fmt(f)?,
                        ExportOrReq::Req(req) => req.fmt(f)?,
                    }
                    f.write_str("\n")?;
                }
                f.write_str("\n}\n")?;
                Ok(())
            }
        }

        for req in Self::AST
            .into_iter()
            .map(|inner| inner.into_iter())
            .flatten()
        {
            exports
                .entry(req.path.ns())
                .or_default()
                .insert(ExportOrReq::Req(*req));
        }

        for export in <Self as RequestTypeVisitor>::dependencies()
            .resolve()
            .into_iter()
        {
            exports
                .entry(export.path.ns())
                .or_default()
                .insert(ExportOrReq::Export(export));
        }

        for export in <Self as ResponseTypeVisitor>::dependencies()
            .resolve()
            .into_iter()
        {
            exports
                .entry(export.path.ns())
                .or_default()
                .insert(ExportOrReq::Export(export));
        }

        if let Some(exports) = exports.remove(Rs::NAME) {
            out.push_str(
                &NamespaceExporter::new(Rs::NAME, exports.into_iter().collect()).to_string(),
            );
        }

        for (name, exports) in exports.into_iter() {
            out.push_str(&NamespaceExporter::new(name, exports.into_iter().collect()).to_string());
        }

        out
    }
}
