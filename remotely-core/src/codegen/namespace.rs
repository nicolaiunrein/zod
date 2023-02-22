type RuntimeValue<T> = &'static (dyn Fn() -> T + Sync);

pub trait Namespace {
    const NAME: &'static str;
    type Req: serde::de::DeserializeOwned;

    fn code() -> String {
        let members =
            || inventory::iter::<NsMember>().filter(|member| member.ns_name() == Self::NAME);

        let member_code = members().map(|member| member.decl()).collect::<String>();

        format!("export namespace {} {{\n{member_code}}};", Self::NAME)
    }
}
pub enum NsMember {
    Interface {
        ns_name: &'static str,
        name: &'static str,
        code: RuntimeValue<String>,
    },
    Method {
        ns_name: &'static str,
        name: &'static str,
        args: RuntimeValue<Vec<(&'static str, String)>>,
        res: RuntimeValue<String>,
    },
}

inventory::collect!(NsMember);

impl NsMember {
    pub fn decl(&self) -> String {
        match self {
            NsMember::Interface {
                ns_name,
                name,
                code,
                ..
            } => {
                let full_name = format!("{ns_name}.{name}");
                let code = (code)();
                let replaced_code = code.replace(&full_name, name);
                format!("export {replaced_code};\n")
            }
            NsMember::Method {
                name,
                args,
                res,
                ns_name,
                ..
            } => {
                let args = (args)();
                let res = (res)();

                let args = args
                    .into_iter()
                    .map(|(name, ty)| format!("{name}: {ty}"))
                    .collect::<Vec<_>>()
                    .join(",");

                format!(
                    "export function {name}({args}): Promise<{res}> {{
                    return request(\"{ns_name}\", \"{name}\", arguments);
                }};"
                )
            }
        }
    }

    pub fn ns_name(&self) -> &str {
        match self {
            NsMember::Interface { ns_name, .. } => ns_name,
            NsMember::Method { ns_name, .. } => ns_name,
        }
    }
}
