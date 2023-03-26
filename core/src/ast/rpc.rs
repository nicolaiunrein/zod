//! Types needed to generate RPC server/client code
use std::fmt::Display;

use crate::{ast::*, Register};

/// TODO
pub trait ClientCodegen {
    fn get() -> String;
}

/// The trait represents a Namespace with rpc methods
pub trait RpcNamespace: crate::Namespace + Register {
    type Req: serde::de::DeserializeOwned;
}

/// The Kind of RpcRequest
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum RpcRequestKind {
    Method,
    Stream,
}

/// This type represents either a remote stream subscription or method call
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RpcRequest {
    pub path: Path,
    pub args: &'static [NamedField],
    pub res: InlineSchema,
    pub kind: RpcRequestKind,
}

impl Display for RpcRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ts_args = self
            .args
            .iter()
            .map(|arg| arg.to_ts_string())
            .collect::<Vec<_>>()
            .join(", ");

        let zod_args = self
            .args
            .iter()
            .map(|arg| arg.value().to_zod_string())
            .collect::<Vec<_>>()
            .join(", ");

        let name = self.path.name();
        let ns = self.path.ns();

        let asyncness = match self.kind {
            RpcRequestKind::Method => "async ",
            RpcRequestKind::Stream => "",
        };

        let res = match self.kind {
            RpcRequestKind::Method => format!("Promise<{}>", self.res.to_ts_string()),
            RpcRequestKind::Stream => format!("Store<{}>", self.res.to_ts_string()),
        };

        let req = match self.kind {
            RpcRequestKind::Method => "request",
            RpcRequestKind::Stream => "stream",
        };

        f.write_str("// @ts-ignore\n")?;

        f.write_fmt(format_args!(
            "export {asyncness}function {name}({ts_args}): {res} {{\n"
        ))?;

        f.write_str("    // phantom usage\n")?;

        for arg in self.args {
            f.write_fmt(format_args!("    {};\n", arg.name()))?;
        }

        f.write_fmt(format_args!(
            "    z.lazy(() => z.tuple([{zod_args}])).parse([...arguments]);\n"
        ))?;

        f.write_fmt(format_args!(
            "    return {req}(\"{ns}\", \"{name}\", arguments);\n"
        ))?;

        f.write_str("};\n")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Namespace;
    use crate::InputType;
    use pretty_assertions::assert_eq;

    #[test]
    fn method_ok() {
        let expected = "\
// @ts-ignore
export async function test(name: Rs.String, age: Rs.U16): Promise<Rs.Option<Rs.Bool>> {
    // phantom usage
    name;
    age;
    z.lazy(() => z.tuple([Rs.String, Rs.U16])).parse([...arguments]);
    return request(\"Ns\", \"test\", arguments);
};
";

        struct Ns;
        impl Namespace for Ns {
            const NAME: &'static str = "Ns";
            const DOCS: Option<Docs> = None;
        }
        const REQ: RpcRequest = RpcRequest {
            path: Path::new::<Ns>("test"),
            kind: RpcRequestKind::Method,
            args: &[
                NamedField::new::<String>("name"),
                NamedField::new::<u16>("age"),
            ],
            res: <Option<bool>>::AST.inline(),
        };

        assert_eq!(REQ.to_string(), expected);
    }

    #[test]
    fn stream_ok() {
        let expected = "\
// @ts-ignore
export function test(name: Rs.String, age: Rs.U16): Store<Rs.Option<Rs.Bool>> {
    // phantom usage
    name;
    age;
    z.lazy(() => z.tuple([Rs.String, Rs.U16])).parse([...arguments]);
    return stream(\"Ns\", \"test\", arguments);
};
";

        struct Ns;
        impl Namespace for Ns {
            const NAME: &'static str = "Ns";
            const DOCS: Option<Docs> = None;
        }
        const REQ: RpcRequest = RpcRequest {
            path: Path::new::<Ns>("test"),
            kind: RpcRequestKind::Stream,
            args: &[
                NamedField::new::<String>("name"),
                NamedField::new::<u16>("age"),
            ],
            res: <Option<bool>>::AST.inline(),
        };

        assert_eq!(REQ.to_string(), expected);
    }
}
