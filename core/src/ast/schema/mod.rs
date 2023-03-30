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

pub struct Exported<T> {
    name: &'static str,
    schema: T,
}

impl<T> Exported<T> {
    pub const fn new(name: &'static str, schema: T) -> Self {
        Self { name, schema }
    }
}
