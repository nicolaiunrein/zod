use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;

use crate::{
    ast::rpc, ast::Export, rpc::Request, rpc::ResponseSender, InputTypeVisitor, Namespace,
    OutputTypeVisitor,
};

use crate::types::Rs;

/// a [JoinHandle](tokio::task::JoinHandle) to cancel a stream when it is dropped.
pub struct StreamHandle(tokio::task::JoinHandle<()>);

/// A map of active subscriber ids to the [JoinHandle](tokio::task::JoinHandle) for stream abortion
pub type SubscriberMap = HashMap<usize, StreamHandle>;

impl Drop for StreamHandle {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// This trait represents a collection of Namespaces
#[async_trait::async_trait]
pub trait Backend: InputTypeVisitor + OutputTypeVisitor {
    async fn handle_request(
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
        let mut exports = BTreeMap::<&str, Vec<_>>::new();

        for export in <Self as InputTypeVisitor>::dependencies()
            .resolve()
            .into_iter()
        {
            exports.entry(export.path.ns()).or_default().push(export);
        }

        for export in <Self as OutputTypeVisitor>::dependencies()
            .resolve()
            .into_iter()
        {
            exports.entry(export.path.ns()).or_default().push(export);
        }

        if let Some(exports) = exports.remove(Rs::NAME) {
            out.push_str(&NamespaceExporter::new(Rs::NAME, exports).to_string());
        }

        for (name, exports) in exports.into_iter() {
            out.push_str(&NamespaceExporter::new(name, exports).to_string());
        }

        out
    }
}

struct NamespaceExporter {
    name: &'static str,
    exports: Vec<Export>,
}

impl NamespaceExporter {
    pub fn new(name: &'static str, mut exports: Vec<Export>) -> Self {
        exports.sort_by_key(|export| export.path.name());
        Self { name, exports }
    }
}

impl Display for NamespaceExporter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("export namepace ")?;
        f.write_str(self.name)?;
        f.write_str(" {\n")?;
        for export in self.exports.iter() {
            export.fmt(f)?;
            f.write_str("\n")?;
        }
        f.write_str("\n}\n")?;
        Ok(())
    }
}
