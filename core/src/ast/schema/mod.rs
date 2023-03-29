mod discriminated_union;
mod fields;
mod newtype;
mod object;
mod raw;
mod r#ref;
mod tuple;
mod union;

pub use discriminated_union::*;
pub use fields::*;
pub use newtype::*;
pub use object::*;
pub use r#ref::*;
pub use r#union::*;
pub use raw::*;
pub use tuple::*;

use super::Formatter;

/// Definition of a zod/typescript schema to be exported
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExportSchema {
    Raw(RawSchema),
    Object(ObjectSchema),
    Newtype(NewtypeSchema),
    Tuple(TupleSchema),
    Union(UnionSchema),
    DiscriminatedUnion(DiscriminatedUnionSchema),
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
