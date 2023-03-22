mod external;
mod macros;

pub mod num;
pub use num::*;

use crate::Namespace;

pub struct Rs;

impl Namespace for Rs {
    const NAME: &'static str = "Rs";
    const DOCS: Option<&'static str> = Some("Rust types");
    type UniqueMembers = ();
}
