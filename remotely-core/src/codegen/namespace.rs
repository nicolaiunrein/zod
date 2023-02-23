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
        schema: RuntimeValue<String>,
        type_def: RuntimeValue<String>,
    },
    Method {
        ns_name: &'static str,
        name: &'static str,
        args: RuntimeValue<Vec<(&'static str, String, String)>>,
        res: RuntimeValue<String>,
    },
    Stream {
        ns_name: &'static str,
        name: &'static str,
        args: RuntimeValue<Vec<(&'static str, String, String)>>,
        res: RuntimeValue<String>,
    },
}

inventory::collect!(NsMember);

impl NsMember {
    pub fn decl(&self) -> String {
        match self {
            NsMember::Interface {
                name,
                schema,
                type_def,
                ..
            } => {
                let schema_name = format!("{name}Schema");
                let schema = (schema)();
                let type_def = (type_def)();
                let schema_export = format!("export const {schema_name} = {schema};\n");
                let interface_export = format!("export interface {name} {type_def}");
                format!("{schema_export}\n{interface_export}")
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

                let arg_fields = args
                    .iter()
                    .map(|(name, ty_name, _)| format!("{name}: {ty_name}"))
                    .collect::<Vec<_>>()
                    .join(",");

                let arg_zod = args
                    .iter()
                    .map(|(_, _, zod)| zod.to_owned())
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
            NsMember::Stream {
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
                    .map(|(name, ty_name, _)| format!("{name}: {ty_name}"))
                    .collect::<Vec<_>>()
                    .join(",");

                let arg_zod = args
                    .iter()
                    .map(|(_, _, zod)| zod.to_owned())
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
            NsMember::Interface { ns_name, .. } => ns_name,
            NsMember::Method { ns_name, .. } => ns_name,
            NsMember::Stream { ns_name, .. } => ns_name,
        }
    }
}
