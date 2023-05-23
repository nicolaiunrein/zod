use std::fmt::Display;

use crate::utils::Separated;

use super::{Ts, Zod, ZodType, ZodTypeAny, ZodTypeInner};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct ZodExport {
    pub name: &'static str,
    #[builder(default)]
    pub args: &'static [&'static str],
    #[builder(setter(into))]
    pub value: ZodType,
}

impl Display for Zod<'_, ZodExport> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.args.is_empty() {
            f.write_fmt(format_args!(
                "export const {name} = {value};",
                name = self.name,
                value = Zod(&self.value)
            ))
        } else {
            let args = self
                .args
                .iter()
                .map(|name| format!("{name}: {any}", any = Zod(&ZodTypeAny)))
                .collect::<Vec<_>>();
            f.write_fmt(format_args!(
                "export const {name} = ({args}) => {value};",
                name = self.name,
                args = Separated(", ", &args),
                value = Zod(&self.value)
            ))
        }
    }
}

impl Display for Ts<'_, ZodExport> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.value.inner {
            ZodTypeInner::String(_) | ZodTypeInner::Number(_) | ZodTypeInner::Generic(_) => {}
            ZodTypeInner::Object(ref obj) => {
                if self.args.is_empty() {
                    f.write_fmt(format_args!(
                        "export interface {name} {obj};",
                        name = self.name,
                        obj = Ts(obj),
                    ))?;
                } else {
                    let args = self
                        .args
                        .iter()
                        .map(|name| format!("{name}"))
                        .collect::<Vec<_>>();

                    f.write_fmt(format_args!(
                        "export interface {name}<{args}> {obj};",
                        name = self.name,
                        args = Separated(", ", &args),
                        obj = Ts(obj),
                    ))?;
                }
            }
        }
        Ok(())
    }
}

impl From<ZodExport> for crate::Export {
    fn from(value: ZodExport) -> Self {
        Self {
            ts: Ts(&value).to_string(),
            zod: Zod(&value).to_string(),
        }
    }
}
