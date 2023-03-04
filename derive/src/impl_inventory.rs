use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Path};

use crate::get_zod;

pub fn expand(ident: &Ident, ns_path: &Path, name: &str) -> TokenStream {
    let zod = get_zod();
    quote! {
        #zod::__private::inventory::submit!(#zod::NamespaceMemberDefinition::new_for::<#ident>(<#ns_path as #zod::Namespace>::NAME, #name));
    }
}
