mod discriminated_union;
mod fields;
mod newtype;
mod object;
mod tuple;
mod union;

pub use discriminated_union::*;
pub use fields::*;
pub use newtype::*;
pub use object::*;
pub use r#union::*;
pub use tuple::*;

use std::fmt::Display;

use super::{Delimited, Formatter, GenericArgument, Path};

/// Definition of a zod/typescript schema to be exported
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExportSchema {
    Raw {
        args: &'static [GenericArgument],
        ts: &'static str,
        zod: &'static str,
    },
    Object(ObjectSchema),
    Newtype(NewtypeSchema),
    Tuple(TupleSchema),
    Union(UnionSchema),
    DiscriminatedUnion(DiscriminatedUnionSchema),
}

/// Definition of a zod/typescript schema to be inlined into other definitions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InlineSchema {
    Ref {
        path: Path,
        args: &'static [InlineSchema],
    },
    Object(ObjectSchema),
    Newtype(NewtypeSchema),
    Tuple(TupleSchema),
    Union(UnionSchema),
    DiscriminatedUnion(DiscriminatedUnionSchema),
}

impl InlineSchema {
    pub const fn path(&self) -> Option<Path> {
        match self {
            InlineSchema::Ref { path, .. } => Some(*path),
            _ => None,
        }
    }
}

impl Formatter for InlineSchema {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InlineSchema::Ref { path, args } => {
                path.fmt(f)?;
                if !args.is_empty() {
                    f.write_str("(")?;
                    args.iter().comma_separated(f, |f, arg| arg.fmt_zod(f))?;

                    f.write_str(")")?;
                }
            }
            InlineSchema::Object(typed) => typed.fmt_zod(f)?,
            InlineSchema::Newtype(typed) => typed.fmt_zod(f)?,
            InlineSchema::Tuple(typed) => typed.fmt_zod(f)?,
            InlineSchema::Union(typed) => typed.fmt_zod(f)?,
            InlineSchema::DiscriminatedUnion(typed) => typed.fmt_zod(f)?,
        }
        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InlineSchema::Ref { path, args } => {
                path.fmt(f)?;
                if !args.is_empty() {
                    f.write_str("<")?;
                    args.iter().comma_separated(f, |f, arg| arg.fmt_ts(f))?;
                    f.write_str(">")?;
                }
            }
            InlineSchema::Object(typed) => typed.fmt_ts(f)?,
            InlineSchema::Newtype(typed) => typed.fmt_ts(f)?,
            InlineSchema::Tuple(typed) => typed.fmt_ts(f)?,
            InlineSchema::Union(typed) => typed.fmt_ts(f)?,
            InlineSchema::DiscriminatedUnion(typed) => typed.fmt_ts(f)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::NamedField;
    use crate::RequestType;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn tuple_ok() {
        const DEF: TupleSchema = TupleSchema::new(&[
            TupleField::new::<String>(),
            TupleField::new::<crate::types::Usize>(),
        ]);
        assert_eq!(DEF.to_zod_string(), "z.tuple([Rs.String, Rs.Usize])");
        assert_eq!(DEF.to_ts_string(), "[Rs.String, Rs.Usize]");
    }

    #[test]
    fn object_ok() {
        const DEF: ObjectSchema = ObjectSchema::new(&[
            NamedField::new::<String>("a"),
            NamedField::new::<crate::types::Usize>("b"),
        ]);

        assert_eq!(
            DEF.to_zod_string(),
            "z.object({ a: Rs.String, b: Rs.Usize })"
        );
        assert_eq!(DEF.to_ts_string(), "{ a: Rs.String, b: Rs.Usize }");
    }

    #[test]
    fn union_ok() {
        const DEF: UnionSchema =
            UnionSchema::new(&[String::AST.inline(), crate::types::Usize::AST.inline()]);

        assert_eq!(DEF.to_zod_string(), "z.union([Rs.String, Rs.Usize])");
        assert_eq!(DEF.to_ts_string(), "Rs.String | Rs.Usize");
    }

    #[test]
    fn discriminated_union_ok() {
        const FIELDS: &[ObjectSchema] = &[
            ObjectSchema::new(&[
                NamedField::new::<String>("myKey"),
                NamedField::new::<crate::types::Usize>("b"),
            ]),
            ObjectSchema::new(&[
                NamedField::new::<String>("myKey"),
                NamedField::new::<crate::types::Isize>("c"),
            ]),
        ];

        const DEF: DiscriminatedUnionSchema = DiscriminatedUnionSchema::new("myKey", FIELDS);
        assert_eq!(
            DEF.to_zod_string(),
            format!(
                "z.discriminatedUnion(\"myKey\", [{}, {}])",
                FIELDS[0].to_zod_string(),
                FIELDS[1].to_zod_string()
            )
        );
        assert_eq!(
            DEF.to_ts_string(),
            format!(
                "{} | {}",
                FIELDS[0].to_ts_string(),
                FIELDS[1].to_ts_string()
            )
        );
    }
}
