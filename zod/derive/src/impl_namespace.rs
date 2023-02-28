use crate::args;
use quote::quote;

pub fn expand(
    input: args::NamespaceInput,
    docs: crate::docs::RustDocs,
) -> proc_macro2::TokenStream {
    let ident = input.ident;
    let name = input.name.unwrap_or_else(|| ident.to_string());
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics ::zod::Namespace for #ident #ty_generics #where_clause {
            const NAME: &'static str = #name;

            fn docs() -> Option<&'static str> {
                Some(#docs)
            }
        }
    }
}
