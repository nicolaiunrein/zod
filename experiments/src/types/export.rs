use std::fmt::Display;

use crate::{types::Crate, utils::Separated};

use super::{Ts, Zod, ZodType, ZodTypeAny, ZodTypeInner};
use quote::{quote, ToTokens};
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
            ZodTypeInner::Arg(_) => todo!(),
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

impl ToTokens for ZodExport {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.name;
        let args = self.args;
        let value = &self.value;

        tokens.extend(quote!(#Crate::types::ZodExport {
            name: #name,
            args: &[#(#args),*],
            value: #value
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::{
        test_utils::{expand_zod, formatted},
        types::{ZodNumber, ZodObject, ZodObjectField, ZodString},
    };
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn codegen() {
        let export = ZodExport::builder()
            .name("Test")
            .args(&["T1", "T2", "T3"])
            .value(
                ZodObject::builder()
                    .fields(vec![
                        ZodObjectField::builder()
                            .name("my_string")
                            .value(ZodString)
                            .build(),
                        ZodObjectField::builder()
                            .name("my_number")
                            .value(ZodNumber)
                            .build(),
                    ])
                    .build(),
            )
            .build();

        assert_eq!(
            formatted(export),
            formatted(expand_zod(quote!(crate::types::ZodExport {
                name: "Test",
                args: &["T1", "T2", "T3"],
                value: crate::types::ZodType {
                    optional: false,
                    inner: crate::types::ZodTypeInner::Object(crate::types::ZodObject {
                        fields: vec![
                            crate::types::ZodObjectField {
                                name: "my_string",
                                value: crate::types::ZodType {
                                    optional: false,
                                    inner: crate::types::ZodTypeInner::String(
                                        crate::types::ZodString
                                    )
                                }
                            },
                            crate::types::ZodObjectField {
                                name: "my_number",
                                value: crate::types::ZodType {
                                    optional: false,
                                    inner: crate::types::ZodTypeInner::Number(
                                        crate::types::ZodNumber
                                    )
                                }
                            }
                        ]
                    })
                }
            })))
        )
    }
}
