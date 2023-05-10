//! Types needed to generate RPC server/client code
use std::fmt::Display;

use crate::ast::*;

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
    pub output: Ref,
    pub kind: RpcRequestKind,
}

impl RpcRequest {
    pub fn is_stream(&self) -> bool {
        matches!(self.kind, RpcRequestKind::Stream)
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
            .map(|arg| arg.value().to_zod_string())
            .collect::<Vec<_>>()
            .join(", ");

        let arg_names = self
            .args
            .iter()
            .map(|arg| arg.name())
            .collect::<Vec<_>>()
            .join(", ");

        let name = self.path.name();
        let ns = self.path.ns();

        let asyncness = match self.kind {
            RpcRequestKind::Method => "async ",
            RpcRequestKind::Stream => "",
        };

        let inner_res_ts = self.output.to_ts_string();
        let inner_res_zod = self.output.to_zod_string();

        let res = match self.kind {
            RpcRequestKind::Method => format!("Promise<{}>", inner_res_ts),
            RpcRequestKind::Stream => format!("Rs.Stream<{}>", inner_res_ts),
        };

        f.write_str("// @ts-ignore\n")?;

        f.write_fmt(format_args!("{asyncness} {name}({ts_args}): {res} {{\n"))?;

        f.write_fmt(format_args!(
            "    z.lazy(() => z.tuple([{zod_args}])).parse([{arg_names}]);\n"
        ))?;

        match self.kind {
            RpcRequestKind::Method => {
                f.write_fmt(format_args!(
                    "    return {inner_res_zod}.parse(await client.call(\"{ns}\", \"{name}\", [{arg_names}]));\n"
                ))?;
            }
            RpcRequestKind::Stream => {
                f.write_fmt(format_args!(
                    "    return {{
      subscribe(cb) {{
        return client
          .get_stream(\"{ns}\", \"{name}\", [{arg_names}])
          .subscribe((val) => {{
            cb({inner_res_zod}.parse(val));
          }});
      }}
}}"
                ))?;
            }
        }

        f.write_str("}")?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Namespace;
    use pretty_assertions::assert_eq;

    #[test]
    fn method_ok() {
        let expected = "\
// @ts-ignore
async test(name: Rs.String, age: Rs.U16): Promise<Rs.Option<Rs.Bool>> {
    // phantom usage
    name;
    age;
    z.lazy(() => z.tuple([Rs.String, Rs.U16])).parse([...arguments]);
    return call(\"Ns\", \"test\", arguments);
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
                NamedField::new_req::<String>("name"),
                NamedField::new_req::<u16>("age"),
            ],
            output: Ref::new_res::<Option<bool>>(),
        };

        assert_eq!(REQ.to_string(), expected);
    }

    #[test]
    fn stream_ok() {
        let expected = "\
// @ts-ignore
test(name: Rs.String, age: Rs.U16): Rs.Stream<Rs.Option<Rs.Bool>> {
    // phantom usage
    name;
    age;
    z.lazy(() => z.tuple([Rs.String, Rs.U16])).parse([...arguments]);
    return subscribe(\"Ns\", \"test\", arguments);
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
                NamedField::new_req::<String>("name"),
                NamedField::new_req::<u16>("age"),
            ],
            output: Ref::new_res::<Option<bool>>(),
        };

        assert_eq!(REQ.to_string(), expected);
    }
}
