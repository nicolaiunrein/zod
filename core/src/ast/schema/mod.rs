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

use crate::{RequestType, ResponseType};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Ref {
    path: Path,
    args: &'static [Ref],
}

impl Ref {
    pub const fn new_req<T: RequestType>() -> Self {
        let path = T::EXPORT.path;
        let args = T::ARGS;

        Self { path, args }
    }

    pub const fn new_res<T: ResponseType>() -> Self {
        let path = T::EXPORT.path;
        let args = T::ARGS;

        Self { path, args }
    }
}

impl Formatter for Ref {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.path, f)?;
        if !self.args.is_empty() {
            f.write_str("(")?;
            self.args
                .iter()
                .comma_separated(f, |f, arg| arg.fmt_zod(f))?;

            f.write_str(")")?;
        }

        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.path, f)?;
        if !self.args.is_empty() {
            f.write_str("<")?;
            self.args
                .iter()
                .comma_separated(f, |f, arg| arg.fmt_ts(f))?;
            f.write_str(">")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::types::{Isize, Usize};

    use super::NamedField;

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
            NamedField::new("a", Ref::new_req::<String>()),
            NamedField::new("b", Ref::new_req::<Usize>()),
        ]);

        assert_eq!(
            DEF.to_zod_string(),
            "z.object({ a: Rs.String, b: Rs.Usize })"
        );
        assert_eq!(DEF.to_ts_string(), "{ a: Rs.String, b: Rs.Usize }");
    }

    #[test]
    fn union_ok() {
        const DEF: UnionSchema = UnionSchema::new(&[
            Ref::new_req::<String>(),
            Ref::new_req::<crate::types::Usize>(),
        ]);

        assert_eq!(DEF.to_zod_string(), "z.union([Rs.String, Rs.Usize])");
        assert_eq!(DEF.to_ts_string(), "Rs.String | Rs.Usize");
    }

    #[test]
    fn discriminated_union_ok() {
        const FIELDS: &[ObjectSchema] = &[
            ObjectSchema::new(&[
                NamedField::new("myKey", Ref::new_req::<String>()),
                NamedField::new("b", Ref::new_req::<Usize>()),
            ]),
            ObjectSchema::new(&[
                NamedField::new("myKey", Ref::new_req::<String>()),
                NamedField::new("c", Ref::new_req::<Isize>()),
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
