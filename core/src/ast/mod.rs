//! We cannot inline partially resolved types.
//! # Example
//! ```rust,ignore
//! struct Generic<T1, T2> {
//!   t1: T1,
//!   t2: T2,
//! }
//!
//! struct MyType<T> {
//!     inner: Generic<String, T>
//! }
//! ```
//!
//! Here `MyType<T>` cannot be exported as `const MyType = (T: z.ZodTypeAny) => z.object({ inner:
//! Ns.Generic(z.String, T) })`
//!
//!

pub(crate) mod build_ins;
mod formatter;
mod utils;

pub use formatter::*;
use std::fmt::Display;
pub use utils::*;

pub trait Node {
    const PATH: Path;
    fn export() -> Option<Export> {
        None
    }

    fn inline() -> InlineSchema;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Export {
    docs: Option<&'static str>,
    path: Path,
    schema: Schema,
}

impl Export {
    pub const fn ns(&self) -> &'static str {
        self.path.ns
    }
    pub fn schema(&self) -> &Schema {
        &self.schema
    }
}

impl Display for Export {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_ts(f)?;
        f.write_str("\n")?;
        self.fmt_zod(f)?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GenericArgument {
    Type(&'static str),
    Const {
        name: &'static str,
        path: Path,
    },
    Assign {
        name: &'static str,
        value: &'static str,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Schema {
    Raw {
        args: &'static [GenericArgument],
        ts: &'static str,
        zod: &'static str,
    },
    Object(Vec<NamedField>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InlineSchema {
    Ref(Path),
    Generic { path: Path, args: Vec<InlineSchema> },
    Object(Vec<NamedField>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamedField {
    name: &'static str,
    value: InlineSchema,
}

impl NamedField {
    pub fn new<T: Node>(name: &'static str) -> Self {
        Self {
            name,
            value: T::inline(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Path {
    ns: &'static str,
    name: &'static str,
}

impl<'a> Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ns)?;
        f.write_str(".")?;
        f.write_str(self.name)?;
        Ok(())
    }
}

impl Formatter for GenericArgument {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericArgument::Type(name) => {
                f.write_str(name)?;
                f.write_str(": ")?;
                f.write_str("z.ZodTypeAny")?;
            }

            GenericArgument::Const { name, path } => {
                f.write_str(name)?;
                f.write_str(": ")?;
                path.fmt(f)?;
            }
            GenericArgument::Assign { .. } => {}
        }
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericArgument::Type(name) => f.write_str(name),
            GenericArgument::Assign { name, value } => {
                f.write_str(name)?;
                f.write_str(" = ")?;
                f.write_str(value)?;
                Ok(())
            }
            GenericArgument::Const { name, path } => {
                f.write_str(name)?;
                f.write_str(" extends ")?;
                path.fmt(f)?;
                Ok(())
            }
        }
    }
}

impl Formatter for InlineSchema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InlineSchema::Ref(path) => {
                path.fmt(f)?;
            }
            InlineSchema::Generic { path, args } => {
                path.fmt(f)?;
                f.write_str("(")?;
                args.iter().comma_separated(f, |f, arg| arg.fmt_zod(f))?;

                f.write_str(")")?;
            }
            InlineSchema::Object(fields) => {
                f.write_str("z.object({ ")?;
                fields
                    .iter()
                    .comma_separated(f, |f, field| field.fmt_zod(f))?;

                f.write_str(" })")?;
            }
        }
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InlineSchema::Ref(path) => path.fmt(f)?,
            InlineSchema::Generic { path, args } => {
                path.fmt(f)?;
                if !args.is_empty() {
                    f.write_str("<")?;
                    args.iter().comma_separated(f, |f, arg| arg.fmt_ts(f))?;
                    f.write_str(">")?;
                }
            }
            InlineSchema::Object(fields) => {
                f.write_str("{ ")?;
                fields
                    .iter()
                    .comma_separated(f, |f, field| field.fmt_ts(f))?;
                f.write_str(" }")?;
            }
        }
        Ok(())
    }
}

impl Formatter for Schema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Schema::Raw { args, zod, .. } => {
                if !args.is_empty() {
                    f.write_str("(")?;
                    args.iter()
                        .filter(|arg| !matches!(arg, GenericArgument::Assign { .. }))
                        .comma_separated(f, |f, arg| arg.fmt_zod(f))?;
                    f.write_str(") => ")?;
                }
                f.write_str(zod)?;
            }
            Schema::Object(fields) => {
                f.write_str("z.object({ ")?;
                fields
                    .iter()
                    .comma_separated(f, |f, field| field.fmt_zod(f))?;

                f.write_str(" })")?;
            }
        }
        Ok(())
    }
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Schema::Raw { args, ts, .. } => {
                if !args.is_empty() {
                    f.write_str("<")?;
                    args.iter().comma_separated(f, |f, arg| arg.fmt_zod(f))?;
                    f.write_str("> => ")?;
                }
                f.write_str(ts)?;
            }
            Schema::Object(fields) => {
                f.write_str(" { ")?;
                fields
                    .iter()
                    .comma_separated(f, |f, field| field.fmt_ts(f))?;
                f.write_str(" }")?;
            }
        }
        Ok(())
    }
}

impl Formatter for Export {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(docs) = self.docs {
            f.write_str(docs)?;
        }
        f.write_str("export const ")?;
        f.write_str(self.path.name)?;
        f.write_str(" = z.lazy(() => ")?;
        self.schema.fmt_zod(f)?;
        f.write_str(");")?;
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(docs) = self.docs {
            f.write_str(docs)?;
        }

        f.write_str("export ")?;
        match self.schema {
            Schema::Raw { ref args, ts, .. } => {
                f.write_str("type ")?;
                f.write_str(self.path.name)?;
                if !args.is_empty() {
                    f.write_str("<")?;
                    args.iter().comma_separated(f, |f, arg| arg.fmt_ts(f))?;
                    f.write_str(">")?;
                }
                f.write_str(" = ")?;
                f.write_str(ts)?;
                f.write_str(";")?;
            }
            Schema::Object(_) => {
                f.write_str("interface ")?;
                f.write_str(self.path.name)?;
                self.schema.fmt_ts(f)?;
            }
        }
        Ok(())
    }
}

impl Formatter for NamedField {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)?;
        f.write_str(": ")?;
        self.value.fmt_zod(f)
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)?;
        f.write_str(": ")?;
        self.value.fmt_ts(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    struct MyGeneric<T1, T2> {
        t1: T1,
        t2: T2,
    }

    impl<T1: Node, T2: Node> Node for MyGeneric<T1, T2> {
        const PATH: Path = Path {
            ns: "Ns",
            name: "MyGeneric",
        };
        fn inline() -> InlineSchema {
            InlineSchema::Generic {
                path: Self::PATH,
                args: vec![T1::inline(), T2::inline()],
            }
        }
    }

    struct MyType {
        inner_my_type: Partial<usize>,
    }

    impl Node for MyType {
        const PATH: Path = Path {
            ns: "Rs",
            name: "MyType",
        };

        fn export() -> Option<Export> {
            Some(Export {
                docs: None,
                path: Self::PATH,
                schema: Schema::Object(vec![NamedField::new::<Partial<usize>>("my_type_inner")]),
            })
        }

        fn inline() -> InlineSchema {
            InlineSchema::Ref(Path {
                ns: "Ns",
                name: "MyType",
            })
        }
    }

    struct Partial<T> {
        partial_inner: MyGeneric<String, T>,
    }

    impl<T: Node> Node for Partial<T> {
        const PATH: Path = Path {
            ns: "Custom",
            name: "Partial",
        };

        fn inline() -> InlineSchema {
            InlineSchema::Object(vec![NamedField::new::<MyGeneric<String, T>>(
                "partial_inner",
            )])
        }
    }

    #[test]
    fn nested_ok() {
        let export = <MyType>::export();
        let expected_zod_export= "export const MyType = z.lazy(() => z.object({ my_type_inner: z.object({ partial_inner: Ns.MyGeneric(Rs.String, Rs.Usize) }) }));";
        let expected_ts_export = "export interface MyType { my_type_inner: { partial_inner: Ns.MyGeneric<Rs.String, Rs.Usize> } }";
        assert_eq!(
            export.as_ref().unwrap().to_zod_string(),
            expected_zod_export
        );

        assert_eq!(export.as_ref().unwrap().to_ts_string(), expected_ts_export);
    }
}
