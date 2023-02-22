use std::collections::{BTreeSet, HashSet};

type RuntimeValue<T> = &'static (dyn Fn() -> T + Sync);

pub trait Namespace {
    const NAME: &'static str;
    type Req: serde::de::DeserializeOwned;

    fn code() -> String {
        let members =
            || inventory::iter::<NsMember>().filter(|member| member.ns_name() == Self::NAME);

        let member_code = members().map(|member| member.decl()).collect::<String>();

        let dependencies: HashSet<String> = members().flat_map(|m| m.deps()).collect();

        let imports = dependencies
            .iter()
            .map(|dep| format!("import {{ {dep} }} from \"./{dep}\";\n"))
            .collect::<String>();

        let export = format!("export namespace {} {{\n{member_code}}};", Self::NAME);

        format!("{imports}{export}")
    }
}
pub enum NsMember {
    Interface {
        ns_name: &'static str,
        name: &'static str,
        raw_decl: RuntimeValue<String>,
        raw_deps: RuntimeValue<Vec<ts_rs::Dependency>>,
    },
    Method {
        ns_name: &'static str,
        name: &'static str,
        args: RuntimeValue<Vec<(&'static str, String)>>,
        res: RuntimeValue<String>,
        raw_deps: RuntimeValue<Vec<ts_rs::Dependency>>,
    },
}

inventory::collect!(NsMember);

impl NsMember {
    pub fn decl(&self) -> String {
        match self {
            NsMember::Interface {
                ns_name,
                name,
                raw_decl,
                ..
            } => {
                let raw = (raw_decl)();
                let full_name = format!("{ns_name}.{name}");
                let decl = raw.replace(&full_name, name);
                format!("export {decl};\n")
            }
            NsMember::Method {
                name, args, res, ..
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
                    return request(\"Watchout\", \"hello\", arguments);
                }};"
                )
            }
        }
    }

    pub fn deps(&self) -> BTreeSet<String> {
        match self {
            NsMember::Interface { raw_deps, .. } | NsMember::Method { raw_deps, .. } => {
                (raw_deps)()
                    .into_iter()
                    .map(|dep| {
                        dep.ts_name
                            .split_once('.')
                            .expect("dependency has shape `namespace.name`")
                            .0
                            .to_string()
                    })
                    .collect()
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
