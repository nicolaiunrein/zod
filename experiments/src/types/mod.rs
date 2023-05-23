mod export;
mod number;
mod object;
mod string;
mod r#type;

pub use export::*;
pub use number::*;
pub use object::*;
pub use r#type::*;
pub use string::*;

use std::{fmt::Display, ops::Deref};

pub struct ZodTypeAny;

impl Display for Zod<'_, ZodTypeAny> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("z.ZodTypeAny")
    }
}

pub struct Zod<'a, T>(pub &'a T);
pub struct Ts<'a, T>(pub &'a T);

impl<T> Deref for Zod<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<T> Deref for Ts<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl From<ZodObject> for ZodTypeInner {
    fn from(value: ZodObject) -> Self {
        Self::Object(value)
    }
}
