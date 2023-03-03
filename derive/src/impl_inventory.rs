use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path};

pub fn expand(ident: &Ident, ns_path: &Path, name: &str) -> TokenStream {
    quote! {
        ::zod::__private::inventory::submit!(::zod::NamespaceMemberDefinition::new_for::<#ident>(<#ns_path as ::zod::Namespace>::NAME, #name));
    }
}
