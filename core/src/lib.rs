//! **_NOTE:_**  This crate is not ready for production yet!

#![deny(unsafe_code)]

#[cfg(feature = "rpc")]
pub mod rpc;

mod build_ins;

use std::any::TypeId;

pub use build_ins::*;

#[derive(Clone, Copy)]
pub struct Code {
    pub ns: &'static (dyn Fn() -> TypeId + Sync),
    pub name: &'static str,
    pub type_def: &'static str,
    pub schema: &'static str,
}

impl Code {
    pub const fn new<T: 'static>(
        name: &'static str,
        type_def: &'static str,
        schema: &'static str,
    ) -> Self {
        Self {
            ns: &|| TypeId::of::<T>(),
            name,
            type_def,
            schema,
        }
    }

    pub fn is_member_of<T: ?Sized + 'static>(&self) -> bool {
        TypeId::of::<T>() == (&self.ns)()
    }
}

inventory::collect!(Code);

impl Code {
    fn ns_name(&self) -> Option<&'static str> {
        self.name.split('.').next()
    }
}

pub trait ZodType {
    const CODE: Code;
}

pub trait Namespace {
    const NAME: &'static str;
    const DOCS: Option<&'static str>;
    fn members() -> Vec<Code>
    where
        Self: 'static,
    {
        let all = inventory::iter::<Code>();
        all.filter(|code| code.is_member_of::<Self>())
            .map(|code| *code)
            .collect()
    }
}
