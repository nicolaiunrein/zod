use crate::Register;

use super::{Export, InlineSchema};

pub trait Node: Register {
    const DEFINITION: Definition;

    fn export() -> Option<Export> {
        Self::DEFINITION.export
    }

    fn inline() -> InlineSchema {
        Self::DEFINITION.inline
    }
}

#[derive(Clone, Copy)]
pub struct Definition {
    pub export: Option<Export>,
    pub inline: InlineSchema,
}
