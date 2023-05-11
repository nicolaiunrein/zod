use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Display;

use crate::ast::rpc::RpcRequest;
use crate::{
    ast::Export, rpc::Request, rpc::ResponseSender, Namespace, RequestTypeVisitor,
    ResponseTypeVisitor,
};

use crate::types::Rs;

const STATIC_TYPE_DEFS: &str = r#"

    export const StreamResponse = z.object({
        stream: z.object({
            id: z.coerce.bigint().nonnegative().lt(2n ** 64n),
            data: z.unknown()
        }),
    });
    export type StreamResponse = z.infer<typeof StreamResponse>;

    export const MethodResponse = z.object({
        method: z.object({
            id: z.coerce.bigint().nonnegative().lt(2n ** 64n),
            data: z.unknown()
        })
    });
    export type MethodResponse = z.infer<typeof MethodResponse>

    export const ErrorResponse =
        z.object({
            error: z.object({
                id: z.coerce.bigint().nonnegative().lt(2n ** 64n).optional(),
                data: z.unknown()
            })
        });

    export type ErrorResponse = z.infer<typeof ErrorResponse>

    export const Response = z.union([
        StreamResponse,
        MethodResponse,
        ErrorResponse
    ])

    export interface Client {
      get_stream(ns: string, method: string, args: unknown[]): Stream<unknown>;
      call(ns: string, method: string, args: unknown[]): Promise<unknown>;
    }
    export interface Stream<T> {
      subscribe(
        next: (value: StreamEvent<T>) => void
      ): () => void;
    }
  
    export type StreamEvent<T> = { data: T } | { error: ZodError } | { loading: true };
  
    export interface ZodError {
      kind: "JsonError",
      msg: string
    }
"#;

/// a [JoinHandle](tokio::task::JoinHandle) to cancel a stream when it is dropped.
pub struct StreamHandle(tokio::task::JoinHandle<()>);

impl From<tokio::task::JoinHandle<()>> for StreamHandle {
    fn from(inner: tokio::task::JoinHandle<()>) -> Self {
        Self(inner)
    }
}

/// A map of active subscriber ids to the [JoinHandle](tokio::task::JoinHandle) for stream abortion
pub type SubscriberMap = HashMap<(usize, usize), StreamHandle>;

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
        connection_id: usize,
        req: Request,
        res: ResponseSender,
        subscribers: &mut SubscriberMap,
    );

    fn generate() -> String
    where
        Self: 'static,
    {
        let mut out = String::from("import { z } from \"zod\";\n\n");
        let mut exports = BTreeMap::<&str, (HashSet<Export>, HashSet<RpcRequest>)>::new();

        struct NamespaceExporter {
            name: &'static str,
            exports: Vec<Export>,
            requests: Vec<RpcRequest>,
            is_rs: bool,
        }

        impl NamespaceExporter {
            pub fn new_rs((exports, requests): (HashSet<Export>, HashSet<RpcRequest>)) -> Self {
                let mut exports: Vec<_> = exports.into_iter().collect();
                let mut requests: Vec<_> = requests.into_iter().collect();

                exports.sort_by(|a, b| a.path.name().cmp(b.path.name()));
                requests.sort_by(|a, b| a.path.name().cmp(b.path.name()));

                Self {
                    name: Rs::NAME,
                    exports,
                    requests,
                    is_rs: true,
                }
            }
            pub fn new(
                name: &'static str,
                (exports, requests): (HashSet<Export>, HashSet<RpcRequest>),
            ) -> Self {
                let mut exports: Vec<_> = exports.into_iter().collect();
                let mut requests: Vec<_> = requests.into_iter().collect();

                exports.sort_by(|a, b| a.path.name().cmp(b.path.name()));
                requests.sort_by(|a, b| a.path.name().cmp(b.path.name()));

                Self {
                    name,
                    exports,
                    requests,
                    is_rs: false,
                }
            }
        }

        impl Display for NamespaceExporter {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("export namespace ")?;
                f.write_str(self.name)?;
                f.write_str(" {\n")?;
                for export in self.exports.iter() {
                    export.fmt(f)?;
                    f.write_str("\n\n")?;
                }

                if !self.requests.is_empty() {
                    f.write_str("export function init(client: Rs.Client)")?;
                    f.write_str("{")?;

                    f.write_str("return {")?;
                    for req in self.requests.iter() {
                        req.fmt(f)?;
                        f.write_str(",\n\n")?;
                    }
                    f.write_str("}}")?;
                }

                if self.is_rs {
                    f.write_str(STATIC_TYPE_DEFS)?;
                }

                f.write_str("\n}\n\n")?;
                Ok(())
            }
        }

        for req in Self::AST.iter().flat_map(|inner| inner.iter()) {
            exports.entry(req.path.ns()).or_default().1.insert(*req);
        }

        for export in <Self as RequestTypeVisitor>::dependencies()
            .resolve()
            .into_iter()
        {
            exports
                .entry(export.path.ns())
                .or_default()
                .0
                .insert(export);
        }

        for export in <Self as ResponseTypeVisitor>::dependencies()
            .resolve()
            .into_iter()
        {
            exports
                .entry(export.path.ns())
                .or_default()
                .0
                .insert(export);
        }

        if let Some(exports) = exports.remove(Rs::NAME) {
            out.push_str(&NamespaceExporter::new_rs(exports).to_string());
        }

        for (name, exports) in exports.into_iter() {
            out.push_str(&NamespaceExporter::new(name, exports).to_string());
        }

        out
    }
}
