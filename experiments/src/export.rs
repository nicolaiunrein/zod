use std::fmt::Display;

use crate::{utils::Separated, IoKind, Kind};

use super::{z, TsFormatter, ZodFormatter, ZodType, ZodTypeInner};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder, Eq, Debug, Clone, Hash)]
pub struct Export<Io> {
    #[builder(setter(into))]
    pub ns: String,
    #[builder(setter(into))]
    pub name: String,

    #[builder(default)]
    pub args: Vec<&'static str>,

    #[builder(setter(into))]
    pub value: ZodType<Io>,
}

impl<Io> Display for ZodFormatter<'_, Export<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.args.is_empty() {
            f.write_fmt(format_args!(
                "export const {name} = {value};",
                name = self.name,
                value = ZodFormatter(&self.value)
            ))
        } else {
            let args = self
                .args
                .iter()
                .map(|name| format!("{name}: {any}", any = ZodFormatter(&z::ZodTypeAny)))
                .collect::<Vec<_>>();
            f.write_fmt(format_args!(
                "export const {name} = ({args}) => {value};",
                name = self.name,
                args = Separated(", ", &args),
                value = ZodFormatter(&self.value)
            ))
        }
    }
}

impl<Io> Display for TsFormatter<'_, Export<Io>>
where
    Io: IoKind,
{
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
                    value = TsFormatter(inner)
                ))?;
            }

            ZodTypeInner::Alias(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = TsFormatter(inner)
                ))?;
            }

            ZodTypeInner::String(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = TsFormatter(inner)
                ))?;
            }
            ZodTypeInner::Number(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = TsFormatter(inner)
                ))?;
            }

            ZodTypeInner::Literal(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = TsFormatter(inner)
                ))?;
            }

            ZodTypeInner::Union(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = TsFormatter(inner)
                ))?;
            }

            ZodTypeInner::DiscriminatedUnion(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = TsFormatter(inner)
                ))?;
            }

            ZodTypeInner::Tuple(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = TsFormatter(inner)
                ))?;
            }

            ZodTypeInner::Bool(ref inner) => {
                f.write_fmt(format_args!(
                    "export type {name} = {value}{or_undefined};",
                    name = self.name,
                    value = TsFormatter(inner)
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
                        obj = TsFormatter(obj)
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
                        obj = TsFormatter(obj)
                    ))?;
                }
            }
        }
        Ok(())
    }
}

impl From<Export<Kind::Input>> for Export<Kind::EitherIo> {
    fn from(other: Export<Kind::Input>) -> Self {
        Self {
            ns: other.ns,
            name: other.name,
            args: other.args,
            value: other.value.into(),
        }
    }
}

impl From<Export<Kind::Output>> for Export<Kind::EitherIo> {
    fn from(other: Export<Kind::Output>) -> Self {
        Self {
            ns: other.ns,
            name: other.name,
            args: other.args,
            value: other.value.into(),
        }
    }
}

crate::make_eq!(Export {
    ns,
    name,
    args,
    value
});

#[cfg(test)]
mod test {
    use crate::{
        z::{ZodNamedField, ZodObject, ZodString},
        TypeExt,
    };
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn export_object() {
        let export = Export::<Kind::Input>::builder()
            .ns("Ns")
            .name("Test")
            .args(vec!["T1", "T2", "T3"])
            .value(
                ZodObject::builder()
                    .fields(vec![
                        ZodNamedField::builder()
                            .name("my_string")
                            .optional()
                            .value(String::inline())
                            .build(),
                        ZodNamedField::builder()
                            .name("my_number")
                            .value(u8::inline())
                            .build(),
                    ])
                    .build(),
            )
            .build();

        assert_eq!(ZodFormatter(&export).to_string(), "export const Test = (T1: z.ZodTypeAny, T2: z.ZodTypeAny, T3: z.ZodTypeAny) => z.object({ my_string: Rs.input.String.optional(), my_number: Rs.input.U8 });");
        assert_eq!(
            TsFormatter(&export).to_string(),
            "export interface Test<T1, T2, T3> { my_string?: Rs.input.String | undefined, my_number: Rs.input.U8 }"
        )
    }

    #[test]
    fn optional_interface() {
        let export = Export::<Kind::Input>::builder()
            .ns("Ns")
            .name("Test")
            .value(
                ZodType::builder()
                    .optional()
                    .inner(ZodObject::builder().build())
                    .build(),
            )
            .build();

        assert_eq!(TsFormatter(&export).to_string(), "export interface Test {}")
    }

    #[test]
    fn export_string() {
        let export = Export::<Kind::Input>::builder()
            .ns("Ns")
            .name("MyString")
            .value(ZodType::builder().optional().inner(ZodString).build())
            .build();

        assert_eq!(
            ZodFormatter(&export).to_string(),
            "export const MyString = z.string().optional();"
        );

        assert_eq!(
            TsFormatter(&export).to_string(),
            "export type MyString = string | undefined;"
        );
    }
}
