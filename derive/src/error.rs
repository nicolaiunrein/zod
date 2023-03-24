use syn::Type;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("conflicting serde arguments: {0}")]
    SerdeConflict(#[from] SerdeConflict),

    #[error("Input and Output names must match")]
    TransparentMismatch { serde: bool, zod: bool },

    #[error("todo")]
    NoSerde,
}

#[derive(thiserror::Error, Debug)]
pub enum SerdeConflict {
    #[error("generated input and output types must have the same name. {ser} != {de}")]
    Name { ser: String, de: String },

    #[error("todo")]
    Type {
        from: Option<Type>,
        into: Option<Type>,
    },
}

impl From<Error> for darling::Error {
    fn from(value: Error) -> Self {
        darling::Error::custom(format!("zod: `{}`", value))
    }
}
