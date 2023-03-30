use proc_macro2::Span;

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("todo")]
    NoSerde,

    #[error("todo")]
    NonAsyncReturningDefault(Span),

    #[error("namespace methods are not allowed to have lifetimes")]
    NamespaceLifetimes(Span),

    #[error("expected `&mut self` got `{got}`.")]
    WrongSelf { span: Span, got: &'static str },

    #[error("namespace methods must have a self argument")]
    NoSelf(Span),
}

impl Error {
    pub(crate) fn owned_self(span: Span) -> Self {
        Self::WrongSelf { span, got: "self" }
    }

    pub(crate) fn shared_self(span: Span) -> Self {
        Self::WrongSelf { span, got: "&self" }
    }

    pub(crate) fn mut_self(span: Span) -> Self {
        Self::WrongSelf {
            span,
            got: "mut self",
        }
    }
}

impl From<Error> for darling::Error {
    fn from(value: Error) -> Self {
        darling::Error::custom(format!("zod: `{}`", value))
    }
}

impl From<Error> for syn::Error {
    fn from(_value: Error) -> Self {
        todo!()
    }
}
