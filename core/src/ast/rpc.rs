use std::fmt::Display;

use crate::{ast::*, Register};

pub trait ClientCodegen {
    fn get() -> String;
}

pub trait RpcNamespace: crate::Namespace + Register {
    type Req: serde::de::DeserializeOwned;
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RpcArgument {
    name: &'static str,
    schema: crate::ast::InlineSchema,
}

impl Formatter for RpcArgument {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.schema.fmt_zod(f)
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)?;
        f.write_str(": ")?;
        self.schema.fmt_ts(f)?;
        Ok(())
    }
}

impl RpcArgument {
    pub const fn new<T: crate::ast::Node>(name: &'static str) -> Self {
        Self {
            name,
            schema: <T>::DEFINITION.inline(),
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum RpcRequestKind {
    Method,
    Stream,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct RpcRequest {
    path: Path,
    args: &'static [RpcArgument],
    res: InlineSchema,
    kind: RpcRequestKind,
}

impl RpcRequest {
    pub fn path(self) -> Path {
        self.path
    }
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
            .map(|arg| arg.to_zod_string())
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
            f.write_fmt(format_args!("    {};\n", arg.name))?;
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
    use crate::Namespace;
    use pretty_assertions::assert_eq;

    use super::*;

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
            const DOCS: Option<&'static str> = None;
            type UniqueMembers = ();
        }
        const REQ: RpcRequest = RpcRequest {
            path: Path::new::<Ns>("test"),
            kind: RpcRequestKind::Method,
            args: &[
                RpcArgument::new::<String>("name"),
                RpcArgument::new::<u16>("age"),
            ],
            res: <Option<bool>>::DEFINITION.inline(),
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
            const DOCS: Option<&'static str> = None;
            type UniqueMembers = ();
        }
        const REQ: RpcRequest = RpcRequest {
            path: Path::new::<Ns>("test"),
            kind: RpcRequestKind::Stream,
            args: &[
                RpcArgument::new::<String>("name"),
                RpcArgument::new::<u16>("age"),
            ],
            res: <Option<bool>>::DEFINITION.inline(),
        };

        assert_eq!(REQ.to_string(), expected);
    }
}
