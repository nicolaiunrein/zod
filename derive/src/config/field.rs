use serde_derive_internals::attr;

use crate::error::{Error, SerdeConflict};

#[cfg_attr(test, derive(Default))]
#[derive(Clone)]
pub struct FieldConfig {
    pub default: bool,
    pub name: Option<String>,
}

impl FieldConfig {
    pub fn new(input: &attr::Field) -> Result<Self, Error> {
        let name = {
            let name = input.name();
            let ser = name.serialize_name();
            let de = name.deserialize_name();

            if ser != de {
                return Err(Error::SerdeConflict(SerdeConflict::Name { ser, de }).into());
            } else {
                ser
            }
        };

        Ok(Self {
            default: match input.default() {
                attr::Default::None => false,
                _ => true,
            },
            name: if name.chars().all(|c| c.is_numeric()) {
                None
            } else {
                Some(name)
            },
        })
    }
}
