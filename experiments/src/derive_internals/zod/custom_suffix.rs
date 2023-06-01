use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

pub(crate) struct CustomSuffix {
    pub inner: Option<String>,
}

impl ToTokens for CustomSuffix {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let option = quote!(::std::option::Option);
        let expanded = match self.inner {
            Some(ref suffix) => quote!(#option::Some(::std::string::String::from(#suffix))),
            None => quote!(#option::None),
        };

        tokens.extend(expanded)
    }
}
