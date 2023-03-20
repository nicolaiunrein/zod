use crate::{ast::*, DependencyRegistration};

pub trait ClientCodegen {
    fn get() -> String;
}

type RuntimeValue<T> = &'static (dyn Fn() -> T + Sync);

pub trait RpcNamespace: crate::Namespace + DependencyRegistration {
    type Req: serde::de::DeserializeOwned;
}

pub struct RpcArgument {
    name: &'static str,
    code: ZodNode,
}

impl RpcArgument {
    pub fn new<T: crate::ZodType>(name: &'static str) -> Self {
        Self { name, code: T::AST }
    }
}

#[derive(Clone, Copy)]
pub enum RpcMember {
    Method {
        ns_name: &'static str,
        name: &'static str,
        args: RuntimeValue<Vec<RpcArgument>>,
        res: RuntimeValue<&'static str>,
    },
    Stream {
        ns_name: &'static str,
        name: &'static str,
        args: RuntimeValue<Vec<RpcArgument>>,
        res: RuntimeValue<&'static str>,
    },
}

impl RpcMember {
    pub fn decl(&self) -> String {
        match self {
            RpcMember::Method {
                name,
                args,
                res,
                ns_name,
                ..
            } => {
                let args = (args)();
                let res = (res)();

                let arg_fields = args
                    .iter()
                    .map(|arg| format!("{}: {}.{}", arg.name, arg.code.ns(), arg.code.name()))
                    .collect::<Vec<_>>()
                    .join(",");

                let phantom_arg_names = create_phantom_arg_names(&args);

                let arg_zod = args
                    .iter()
                    .map(|arg| format!("{}.{}", arg.code.ns(), arg.code.name()))
                    .collect::<Vec<_>>()
                    .join(",");

                format!(
                    "
// @ts-ignore
export async function {name}({arg_fields}): Promise<{res}> {{
    {phantom_arg_names}
    z.lazy(() => z.tuple([{arg_zod}])).parse([...arguments]);
    return request(\"{ns_name}\", \"{name}\", arguments);
}};"
                )
            }
            RpcMember::Stream {
                name,
                args,
                res,
                ns_name,
                ..
            } => {
                let args = (args)();
                let res = (res)();

                let arg_fields = args
                    .iter()
                    .map(|arg| format!("{}: {}.{}", arg.name, arg.code.ns(), arg.code.name()))
                    .collect::<Vec<_>>()
                    .join(",");

                let phantom_arg_names = create_phantom_arg_names(&args);

                let arg_zod = args
                    .iter()
                    .map(|arg| format!("{}.{}", arg.code.ns(), arg.code.name()))
                    .collect::<Vec<_>>()
                    .join(",");

                format!(
                    "
// @ts-ignore
export function {name}({arg_fields}): Store<{res}> {{
    {phantom_arg_names}
    z.lazy(() => z.tuple([{arg_zod}])).parse([...arguments]);
    return subscribe(\"{ns_name}\", \"{name}\", arguments);
}};"
                )
            }
        }
    }

    pub fn ns_name(&self) -> &'static str {
        match self {
            RpcMember::Method { ns_name, .. } => ns_name,
            RpcMember::Stream { ns_name, .. } => ns_name,
        }
    }
    pub fn name(&self) -> &'static str {
        match self {
            RpcMember::Method { name, .. } => name,
            RpcMember::Stream { name, .. } => name,
        }
    }
}

fn create_phantom_arg_names(args: &[RpcArgument]) -> String {
    todo!()
    // if args.is_empty() {
    // String::new()
    // } else {
    // let mut out = String::from("// phantom usage\n");
    // for arg in args {
    // out.push_str("    ");
    // let ty = arg.code.ty();
    // // todo
    // // out.push_str(&ty.as_arg().to_ts_string());
    // out.push(';');
    // out.push('\n');
    // }
    // out
    // }
}
