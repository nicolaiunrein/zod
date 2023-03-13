use crate::{args, format_ident_for_registration, get_zod};
use quote::quote;
use syn::parse_quote;

pub fn expand(
    input: args::NamespaceInput,
    docs: crate::docs::RustDocs,
) -> proc_macro2::TokenStream {
    let ident = input.ident;
    let name = input.name.unwrap_or_else(|| ident.to_string());
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let p = parse_quote!(#ident);
    let register_path = format_ident_for_registration(&p);

    let zod = get_zod();
    let vis = input.vis;

    quote! {
        impl #impl_generics #zod::Namespace for #ident #ty_generics #where_clause {
            const NAME: &'static str = #name;

            fn docs() -> Option<&'static str> {
                Some(#docs)
            }
        }

        #[allow(dead_code)]
        #[allow(non_camel_case_types)]
        #vis struct #register_path;
    }
}
