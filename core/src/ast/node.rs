use crate::Register;

use super::{Export, InlineSchema};

pub trait Node: Register {
    const DEFINITION: Definition;

    fn export() -> Option<Export> {
        Self::DEFINITION.export()
    }

    fn inline() -> InlineSchema {
        Self::DEFINITION.inline()
    }
}

#[derive(Clone, Copy)]
pub enum Definition {
    Exported {
        export: Export,
        args: &'static [InlineSchema],
    },

    Inlined(InlineSchema),
}

impl Definition {
    pub const fn exported(export: Export, args: &'static [InlineSchema]) -> Self {
        Self::Exported { export, args }
    }

    pub const fn inlined<T: Node>() -> Self {
        Self::Inlined(T::DEFINITION.inline())
    }

    pub const fn partially_inlined(schema: InlineSchema) -> Self {
        Self::Inlined(schema)
    }

    pub const fn export(self) -> Option<Export> {
        match self {
            Definition::Exported { export, .. } => Some(export),
            Definition::Inlined(_) => None,
        }
    }
    pub const fn inline(self) -> InlineSchema {
        match self {
            Definition::Exported { args, export } => InlineSchema::Ref {
                path: export.path,
                args,
            },
            Definition::Inlined(inline) => inline,
        }
    }
}
