use crate::Register;

use super::{Export, InlineSchema};

pub trait Node: Register {
    fn export() -> Option<Export> {
        None
    }

    fn inline() -> InlineSchema;
}
