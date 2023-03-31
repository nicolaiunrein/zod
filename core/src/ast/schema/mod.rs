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

use super::Compiler;

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

impl ExportSchema {
    pub const fn named_fields(self) -> Option<&'static [NamedField]> {
        match self {
            ExportSchema::Object(schema) => Some(schema.fields()),
            _ => None,
        }
    }

    pub const fn named_fields_or_panic(self) -> &'static [NamedField] {
        match self {
            ExportSchema::Object(schema) => schema.fields(),
            ExportSchema::Raw(_) => panic!("ExportSchema::Raw does not have named fields"),
            ExportSchema::Newtype(_) => panic!("ExportSchema::Newtype does not have named fields"),
            ExportSchema::Tuple(_) => panic!("ExportSchema::Tuple does not have named fields"),
            ExportSchema::Union(_) => panic!("ExportSchema::Union does not have named fields"),
            ExportSchema::DiscriminatedUnion(_) => {
                panic!("ExportSchema::DiscriminatedUnion does not have named fields")
            }
        }
    }
}

pub struct Exported<T> {
    name: &'static str,
    schema: T,
}

impl<T> Exported<T> {
    pub const fn new(name: &'static str, schema: T) -> Self {
        Self { name, schema }
    }
}
