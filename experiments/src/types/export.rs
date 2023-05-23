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
        let or_undefined = if self.value.optional {
            " | undefined"
        } else {
            ""
        };
        match self.value.inner {
            ZodTypeInner::Reference(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    value = Ts(inner),
                    name = self.name
                ))?;
            }

            ZodTypeInner::String(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    value = Ts(inner),
                    name = self.name
                ))?;
            }
            ZodTypeInner::Number(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    value = Ts(inner),
                    name = self.name
                ))?;
            }
            ZodTypeInner::Generic(value) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name
                ))?;
            }

            ZodTypeInner::Object(ref obj) => {
                if self.args.is_empty() {
                    f.write_fmt(format_args!(
                        "export interface {name} {obj}",
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
                        "export interface {name}<{args}> {obj}",
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
    fn export_object() {
        let export = ZodExport::builder()
            .name("Test")
            .args(&["T1", "T2", "T3"])
            .value(
                ZodObject::builder()
                    .fields(vec![
                        ZodObjectField::builder()
                            .name("my_string")
                            .value(ZodType::builder().optional().inner(ZodString).build())
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
            formatted(&export),
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
                                    optional: true,
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
        );

        assert_eq!(Zod(&export).to_string(), "export const Test = (T1: z.ZodTypeAny, T2: z.ZodTypeAny, T3: z.ZodTypeAny) => z.object({ my_string: z.string().optional(), my_number: z.number() });");
        assert_eq!(
            Ts(&export).to_string(),
            "export interface Test<T1, T2, T3> { my_string?: string | undefined, my_number: number }"
        )
    }

    #[test]
    fn optional_interface() {
        let export = ZodExport::builder()
            .name("Test")
            .value(
                ZodType::builder()
                    .optional()
                    .inner(ZodObject::builder().build())
                    .build(),
            )
            .build();

        assert_eq!(Ts(&export).to_string(), "export interface Test {  }")
    }

    #[test]
    fn export_string() {
        let export = ZodExport::builder()
            .name("MyString")
            .value(ZodType::builder().optional().inner(ZodString).build())
            .build();

        assert_eq!(
            formatted(&export),
            formatted(expand_zod(quote!(crate::types::ZodExport {
                name: "MyString",
                args: &[],
                value: crate::types::ZodType {
                    optional: true,
                    inner: crate::types::ZodTypeInner::String(crate::types::ZodString)
                }
            })))
        );

        assert_eq!(
            Zod(&export).to_string(),
            "export const MyString = z.string().optional();"
        );

        assert_eq!(
            Ts(&export).to_string(),
            "export type MyString = string | undefined;"
        );
    }
}
