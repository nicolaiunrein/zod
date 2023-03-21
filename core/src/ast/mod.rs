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

use std::fmt::Display;

trait Delimited<F> {
    type Item;
    fn fmt_delimited(
        self,
        f: &mut std::fmt::Formatter<'_>,
        delim: &'static str,
        func: F,
    ) -> std::fmt::Result;

    fn comma_separated(self, f: &mut std::fmt::Formatter<'_>, func: F) -> std::fmt::Result
    where
        Self: Sized,
    {
        self.fmt_delimited(f, ", ", func)
    }
}

impl<Iter, Item, Func> Delimited<Func> for Iter
where
    Iter: Iterator<Item = Item>,
    Func: Fn(&mut std::fmt::Formatter<'_>, Item) -> std::fmt::Result,
{
    type Item = Item;
    fn fmt_delimited(
        self,
        f: &mut std::fmt::Formatter<'_>,
        delim: &'static str,
        func: Func,
    ) -> std::fmt::Result {
        let mut iter = self.peekable();
        while let Some(item) = iter.next() {
            (func)(f, item)?;
            if iter.peek().is_some() {
                f.write_str(delim)?;
            }
        }
        Ok(())
    }
}

pub struct Path<'a> {
    ns: &'a str,
    name: &'a str,
}

impl<'a> Display for Path<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ns)?;
        f.write_str(".")?;
        f.write_str(self.name)?;
        Ok(())
    }
}

trait Formatter {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;

    fn to_zod_string(&self) -> String
    where
        Self: Sized,
    {
        struct Helper<'a>(&'a dyn Formatter);
        impl Display for Helper<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt_zod(f)
            }
        }

        Helper(self).to_string()
    }

    fn to_ts_string(&self) -> String
    where
        Self: Sized,
    {
        struct Helper<'a>(&'a dyn Formatter);
        impl Display for Helper<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt_ts(f)
            }
        }

        Helper(self).to_string()
    }
}

pub struct Export {
    docs: Option<&'static str>,
    ident: &'static str,
    schema: Schema,
}

pub enum Schema {
    Raw(RawSchema),
    Object(Vec<NamedField>),
}

pub struct RawSchema {
    args: &'static [&'static str],
    ts: &'static str,
    zod: &'static str,
}

pub enum InlineSchema {
    Ref(Path<'static>),
    Generic {
        path: Path<'static>,
        args: Vec<InlineSchema>,
    },
    Object(Vec<NamedField>),
}

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

pub trait Node {
    fn export() -> Option<Export> {
        None
    }

    fn inline() -> InlineSchema;
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
            Schema::Raw(raw) => {
                if !raw.args.is_empty() {
                    f.write_str("(")?;
                    for arg in raw.args.iter() {
                        f.write_str(arg)?;
                        f.write_str(": ")?;
                        f.write_str("z.ZodTypeAny")?;
                    }
                    f.write_str(") => ")?;
                }
                f.write_str(raw.zod)?;
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
            Schema::Raw(raw) => {
                if !raw.args.is_empty() {
                    f.write_str("<")?;
                    for arg in raw.args.iter() {
                        f.write_str(arg)?;
                    }
                    f.write_str("> => ")?;
                }
                f.write_str(raw.ts)?;
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
        f.write_str(self.ident)?;
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
            Schema::Raw(ref raw) => {
                f.write_str("type ")?;
                f.write_str(self.ident)?;
                if !raw.args.is_empty() {
                    f.write_str("<")?;
                    raw.args
                        .iter()
                        .comma_separated(f, |f, arg| f.write_str(arg))?;
                    f.write_str(">")?;
                }
                f.write_str(" = ")?;
                f.write_str(raw.ts)?;
                f.write_str(";")?;
            }
            Schema::Object(_) => {
                f.write_str("interface ")?;
                f.write_str(self.ident)?;
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

    impl Node for String {
        fn export() -> Option<Export> {
            Some(Export {
                docs: None,
                ident: "String",
                schema: Schema::Raw(RawSchema {
                    args: &[],
                    zod: "z.string()",
                    ts: "string",
                }),
            })
        }

        fn inline() -> InlineSchema {
            InlineSchema::Ref(Path {
                ns: "Rs",
                name: "String",
            })
        }
    }

    impl Node for usize {
        fn export() -> Option<Export> {
            Some(Export {
                docs: None,
                ident: "Usize",
                schema: Schema::Raw(RawSchema {
                    args: &[],
                    zod: "z.number().int()",
                    ts: "number",
                }),
            })
        }

        fn inline() -> InlineSchema {
            InlineSchema::Ref(Path {
                ns: "Rs",
                name: "Usize",
            })
        }
    }

    impl<T: Node> Node for Option<T> {
        fn export() -> Option<Export> {
            Some(Export {
                docs: None,
                ident: "Option",
                schema: Schema::Raw(RawSchema {
                    args: &["T"],
                    zod: "T.optional()",
                    ts: "T | undefined",
                }),
            })
        }

        fn inline() -> InlineSchema {
            InlineSchema::Generic {
                path: Path {
                    ns: "Rs",
                    name: "Option",
                },
                args: vec![T::inline()],
            }
        }
    }

    struct MyGeneric<T1, T2> {
        t1: T1,
        t2: T2,
    }

    impl<T1: Node, T2: Node> Node for MyGeneric<T1, T2> {
        fn inline() -> InlineSchema {
            InlineSchema::Generic {
                path: Path {
                    ns: "Ns",
                    name: "MyGeneric",
                },
                args: vec![T1::inline(), T2::inline()],
            }
        }
    }

    struct MyType {
        inner_my_type: Partial<usize>,
    }

    impl Node for MyType {
        fn export() -> Option<Export> {
            Some(Export {
                docs: None,
                ident: "MyType",
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

    #[test]
    fn string_ok() {
        let export = <String>::export();
        let expected_zod_export = "export const String = z.lazy(() => z.string());";
        let expected_ts_export = "export type String = string;";

        assert_eq!(
            export.as_ref().unwrap().to_zod_string(),
            expected_zod_export
        );

        assert_eq!(export.as_ref().unwrap().to_ts_string(), expected_ts_export);
    }

    #[test]
    fn option_ok() {
        let export = <Option<String>>::export();
        let expected_zod_export =
            "export const Option = z.lazy(() => (T: z.ZodTypeAny) => T.optional());";
        let expected_ts_export = "export type Option<T> = T | undefined;";

        assert_eq!(
            export.as_ref().unwrap().to_zod_string(),
            expected_zod_export
        );

        assert_eq!(export.as_ref().unwrap().to_ts_string(), expected_ts_export);
    }
}
