use std::{collections::BTreeMap, fmt::Display};

use crate::{utils::Separated, Alias, IoKind, Kind};

use super::{z, TsFormatter, ZodFormatter, ZodType, ZodTypeInner};
use typed_builder::TypedBuilder;

/// The representation of a type definition in the generated code.
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

    #[builder(setter(into, strip_option), default)]
    pub docs: Option<String>,
}

impl<Io> Display for ZodFormatter<'_, Export<Io>>
where
    Io: IoKind,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.args.is_empty() {
            if let Some(ref docs) = self.docs {
                f.write_str("\n")?;
                for line in docs.lines() {
                    f.write_fmt(format_args!("// {line}\n"))?;
                }
            }
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

            if let Some(ref docs) = self.docs {
                f.write_str("\n")?;
                for line in docs.lines() {
                    f.write_fmt(format_args!("// {line}\n"))?;
                }
            }
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

        if let Some(ref docs) = self.docs {
            f.write_str("\n")?;
            for line in docs.lines() {
                f.write_fmt(format_args!("// {line}\n"))?;
            }
        }
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
            docs: other.docs,
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
            docs: other.docs,
            name: other.name,
            args: other.args,
            value: other.value.into(),
        }
    }
}

crate::make_eq!(Export {
    ns,
    name,
    docs,
    args,
    value
});

/// Representation of all generated code
pub struct ExportMap(BTreeMap<String, NsMap>);

impl ExportMap {
    pub fn new(
        input_exports: impl IntoIterator<Item = Export<Kind::Input>>,
        output_exports: impl IntoIterator<Item = Export<Kind::Output>>,
    ) -> Self {
        let mut out = BTreeMap::<String, NsMap>::new();

        for export in input_exports.into_iter() {
            let ns_map = out.entry(export.ns.clone()).or_insert_with(|| NsMap {
                input: Default::default(),
                output: Default::default(),
                io: Default::default(),
            });

            ns_map.insert_input(export.name.clone(), export);
        }

        for export in output_exports.into_iter() {
            let ns_map = out.entry(export.ns.clone()).or_insert_with(|| NsMap {
                input: Default::default(),
                output: Default::default(),
                io: Default::default(),
            });

            ns_map.insert_output(export.name.clone(), export);
        }

        Self(out)
    }
}

impl Display for ExportMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (ns, inner) in self.0.iter() {
            f.write_fmt(format_args!("export namespace {ns} {{\n{}}}\n", inner))?;
        }
        Ok(())
    }
}

impl Display for NsMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn fmt_part<T>(
            f: &mut std::fmt::Formatter<'_>,
            set: &BTreeMap<String, Export<T>>,
        ) -> std::fmt::Result
        where
            T: IoKind,
        {
            let name = T::NAME;
            if set.is_empty() {
                f.write_fmt(format_args!("    export namespace {name} {{}}\n"))?;
            } else {
                f.write_fmt(format_args!("    export namespace {name} {{\n"))?;
                for export in set.values() {
                    let ts = TsFormatter(export).to_string();
                    for line in ts.lines() {
                        if line.is_empty() {
                            f.write_str("\n")?;
                        } else {
                            f.write_fmt(format_args!("        {}\n", line.trim()))?;
                        }
                    }

                    let zod = ZodFormatter(export).to_string();
                    for line in zod.lines() {
                        if line.is_empty() {
                            f.write_str("\n")?;
                        } else {
                            f.write_fmt(format_args!("        {}\n", line.trim()))?;
                        }
                    }
                }

                f.write_str("    }\n")?;
            }
            std::fmt::Result::Ok(())
        }

        fmt_part(f, &self.input)?;
        fmt_part(f, &self.output)?;
        fmt_part(f, &self.io)?;

        Ok(())
    }
}

struct NsMap {
    input: BTreeMap<String, Export<Kind::Input>>,
    output: BTreeMap<String, Export<Kind::Output>>,
    io: BTreeMap<String, Export<Kind::EitherIo>>,
}

impl NsMap {
    fn insert_input(&mut self, name: String, mut input: Export<Kind::Input>) {
        if let Some(output) = self.output.get_mut(&name) {
            if &mut input == output {
                let merged = Export::<Kind::EitherIo>::from(input.clone());

                let alias = Alias {
                    name: merged.name.clone(),
                    ns: merged.ns.clone(),
                };

                input.value = ZodTypeInner::Alias(alias.clone()).into();
                output.value = ZodTypeInner::Alias(alias).into();
                self.io.insert(name.clone(), merged);
            }
        }
        self.input.insert(name, input);
    }

    fn insert_output(&mut self, name: String, mut output: Export<Kind::Output>) {
        if let Some(input) = self.input.get_mut(&name) {
            if &mut output == input {
                let merged = Export::<Kind::EitherIo>::from(output.clone());

                let alias = Alias {
                    name: merged.name.clone(),
                    ns: merged.ns.clone(),
                };

                output.value = ZodTypeInner::Alias(alias.clone()).into();
                input.value = ZodTypeInner::Alias(alias).into();
                self.io.insert(name.clone(), merged);
            }
        }
        self.output.insert(name, output);
    }
}

#[cfg(test)]
mod test {
    use crate::{
        z::{ZodNamedField, ZodObject, ZodString},
        Namespace, Reference, TypeExt,
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

    #[test]
    fn export_map_ok() {
        struct Ns;

        impl Namespace for Ns {
            const NAME: &'static str = "Ns";
        }
        struct Ns2;
        impl Namespace for Ns2 {
            const NAME: &'static str = "Ns2";
        }
        let map = ExportMap::new(
            [
                Export::builder()
                    .name("hello")
                    .ns(Ns::NAME)
                    .value(ZodTypeInner::Generic(String::from("MyGeneric")))
                    .build(),
                Export::builder()
                    .name("world")
                    .docs("My Docs")
                    .ns(Ns2::NAME)
                    .value(
                        ZodObject::builder()
                            .fields(vec![ZodNamedField::builder()
                                .name("hello")
                                .value(Reference::builder().name("hello").ns(Ns::NAME).build())
                                .build()])
                            .build(),
                    )
                    .build(),
            ],
            [Export::builder()
                .name("hello")
                .ns(Ns::NAME)
                .value(ZodTypeInner::Generic(String::from("MyGeneric")))
                .build()],
        );

        assert_eq!(
            map.to_string().trim(),
            r#"
export namespace Ns {
    export namespace input {
        export type hello = Ns.io.hello;
        export const hello = Ns.io.hello;
    }
    export namespace output {
        export type hello = Ns.io.hello;
        export const hello = Ns.io.hello;
    }
    export namespace io {
        export type hello = MyGeneric;
        export const hello = MyGeneric;
    }
}
export namespace Ns2 {
    export namespace input {

        // My Docs
        export interface world { hello: Ns.input.hello }

        // My Docs
        export const world = z.object({ hello: Ns.input.hello });
    }
    export namespace output {}
    export namespace io {}
}"#
            .trim()
        );
    }
}
