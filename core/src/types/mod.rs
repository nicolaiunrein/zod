//! Implementations of [RequestType](crate::RequestType) as [ReturnType](crate::ReturnType) for foreign types
mod external;
mod macros;

pub mod num;
pub use num::*;

use crate::ast::Docs;
use crate::Namespace;

/// A special Namespace which defines all build in types
pub struct Rs;

impl Namespace for Rs {
    const NAME: &'static str = "Rs";
    const DOCS: Option<Docs> = Some(Docs("Rust types"));
}
