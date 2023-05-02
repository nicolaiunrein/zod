use serde_derive_internals::attr;

use crate::error::Error;

use super::Derive;

#[derive(Clone, Debug)]
pub(crate) struct FieldConfig {
    pub(crate) required: bool,
    pub(crate) name: Option<String>,
    pub(crate) ignored: bool,
    pub(crate) derive: Derive,
    pub(crate) flatten: bool,
}

#[cfg(test)]
impl Default for FieldConfig {
    fn default() -> Self {
        Self {
            required: true,
            name: None,
            ignored: false,
            derive: Default::default(),
            flatten: false,
        }
    }
}

impl FieldConfig {
    pub(crate) fn new(input: &attr::Field, derive: Derive) -> Result<Self, Error> {
        let name = match derive {
            Derive::Request => input.name().deserialize_name(),
            Derive::Response => input.name().serialize_name(),
        };

        let required =
            input.skip_serializing_if().is_none() && matches!(input.default(), attr::Default::None);

        let ignored = match derive {
            Derive::Request => input.skip_deserializing(),
            Derive::Response => input.skip_serializing(),
        };

        Ok(Self {
            ignored,
            required,
            // todo
            name: if name.chars().all(|c| c.is_numeric()) {
                None
            } else {
                Some(name)
            },
            derive,
            flatten: input.flatten(),
        })
    }
}
