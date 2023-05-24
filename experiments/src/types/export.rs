use std::fmt::Display;

use crate::{types::crate_name, utils::Separated};

use super::{Context, Ts, Zod, ZodType, ZodTypeAny, ZodTypeInner};
use quote::{quote, ToTokens};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodExport {
    #[builder(setter(into))]
    pub ns: String,
    #[builder(setter(into))]
    pub name: String,

    pub context: Context,

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
                    name = self.name,
                    value = Ts(inner)
                ))?;
            }

            ZodTypeInner::String(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = Ts(inner)
                ))?;
            }
            ZodTypeInner::Number(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = Ts(inner)
                ))?;
            }

            ZodTypeInner::Literal(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = Ts(inner)
                ))?;
            }

            ZodTypeInner::Union(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = Ts(inner)
                ))?;
            }

            ZodTypeInner::DiscriminatedUnion(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = Ts(inner)
                ))?;
            }

            ZodTypeInner::Tuple(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = Ts(inner)
                ))?;
            }

            ZodTypeInner::Bool(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = Ts(inner)
                ))?;
            }

            ZodTypeInner::Generic(ref value) => {
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
                        obj = Ts(obj)
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
                        obj = Ts(obj)
                    ))?;
                }
            }
        }
        Ok(())
    }
}

impl ToTokens for ZodExport {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let args = self.args;
        let value = &self.value;

        tokens.extend(quote!(#crate_name::types::ZodExport {
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
        types::{ZodNamedField, ZodObject, ZodString},
        OutputType,
    };
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn export_object() {
        let export = ZodExport::builder()
            .ns("Ns")
            .name("Test")
            .context(Context::Io)
            .args(&["T1", "T2", "T3"])
            .value(
                ZodObject::builder()
                    .fields(vec![
                        ZodNamedField::builder()
                            .name("my_string")
                            .optional()
                            .value(String::get_output_ref())
                            .build(),
                        ZodNamedField::builder()
                            .name("my_number")
                            .value(u8::get_output_ref())
                            .build(),
                    ])
                    .build(),
            )
            .build();

        let string_ref = String::get_output_ref();
        let u8_ref = u8::get_output_ref();

        assert_eq!(
            formatted(&export),
            formatted(expand_zod(quote!(crate::types::ZodExport {
                name: "Test",
                args: &["T1", "T2", "T3"],
                value: crate::types::ZodType {
                    optional: false,
                    inner: crate::types::ZodTypeInner::Object(crate::types::ZodObject {
                        fields: vec![
                            crate::types::ZodNamedField {
                                name: "my_string",
                                optional: true,
                                value: #string_ref
                            },
                            crate::types::ZodNamedField {
                                name: "my_number",
                                optional: false,
                                value: #u8_ref
                            }
                        ]
                    })
                }
            })))
        );

        assert_eq!(Zod(&export).to_string(), "export const Test = (T1: z.ZodTypeAny, T2: z.ZodTypeAny, T3: z.ZodTypeAny) => z.object({ my_string: Rs.io.String.optional(), my_number: Rs.io.U8 });");
        assert_eq!(
            Ts(&export).to_string(),
            "export interface Test<T1, T2, T3> { my_string?: Rs.io.String | undefined, my_number: Rs.io.U8 }"
        )
    }

    #[test]
    fn optional_interface() {
        let export = ZodExport::builder()
            .ns("Ns")
            .name("Test")
            .context(Context::Io)
            .value(
                ZodType::builder()
                    .optional()
                    .inner(ZodObject::builder().build())
                    .build(),
            )
            .build();

        assert_eq!(Ts(&export).to_string(), "export interface Test {}")
    }

    #[test]
    fn export_string() {
        let export = ZodExport::builder()
            .ns("Ns")
            .name("MyString")
            .context(Context::Io)
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
