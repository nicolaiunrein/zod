pub trait ClientCodegen {
    fn get() -> String;
}

type RuntimeValue<T> = &'static (dyn Fn() -> T + Sync);

pub trait RpcNamespace: crate::Namespace {
    type Req: serde::de::DeserializeOwned;

    fn rpc_members() -> Vec<&'static RpcMember> {
        inventory::iter::<RpcMember>()
            .filter(|member| member.ns_name() == Self::NAME)
            .collect()
    }
}

pub struct RpcArgument {
    name: &'static str,
    type_def: String,
    schema: String,
}

impl RpcArgument {
    pub fn new<T: crate::ZodType>(name: &'static str) -> Self {
        Self {
            name,
            type_def: T::type_def().to_string(),
            schema: T::schema(),
        }
    }
}

pub enum RpcMember {
    Method {
        ns_name: &'static str,
        name: &'static str,
        args: RuntimeValue<Vec<RpcArgument>>,
        res: RuntimeValue<String>,
    },
    Stream {
        ns_name: &'static str,
        name: &'static str,
        args: RuntimeValue<Vec<RpcArgument>>,
        res: RuntimeValue<String>,
    },
}

inventory::collect!(RpcMember);

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
                    .map(|arg| format!("{}: {}", arg.name, arg.type_def))
                    .collect::<Vec<_>>()
                    .join(",");

                let arg_zod = args
                    .iter()
                    .map(|arg| arg.schema.to_owned())
                    .collect::<Vec<_>>()
                    .join(",");

                format!(
                    "
                    // @ts-ignore
                    export async function {name}({arg_fields}): Promise<{res}> {{
                    z.tuple([{arg_zod}]).parse([...arguments]);
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
                    .map(|arg| format!("{}: {}", arg.name, arg.type_def))
                    .collect::<Vec<_>>()
                    .join(",");

                let arg_zod = args
                    .iter()
                    .map(|arg| arg.schema.to_owned())
                    .collect::<Vec<_>>()
                    .join(",");

                format!(
                    "
                    // @ts-ignore
                    export function {name}({arg_fields}): Store<{res}> {{
                    z.tuple([{arg_zod}]).parse([...arguments]);
                    return subscribe(\"{ns_name}\", \"{name}\", arguments);
                }};"
                )
            }
        }
    }

    pub fn ns_name(&self) -> &str {
        match self {
            RpcMember::Method { ns_name, .. } => ns_name,
            RpcMember::Stream { ns_name, .. } => ns_name,
        }
    }
}

#[doc(hidden)]
/// marker trait for better errors
pub trait RpcHandler {}
