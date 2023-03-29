use crate::config::ContainerConfig;
use proc_macro2::TokenStream;
use quote::quote;
use serde_derive_internals::ast::Variant;

#[allow(dead_code)]
pub struct Enum<'a> {
    pub(crate) variants: &'a [Variant<'a>],
    pub(crate) config: &'a ContainerConfig,
}

impl<'a> Enum<'a> {
    pub fn expand(self) -> TokenStream {
        quote!()
    }
}
