use darling::FromDeriveInput;
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

use crate::utils::zod_core;

#[derive(FromDeriveInput)]
#[darling(
    attributes(zod),
    forward_attrs(allow, doc, cfg),
    supports(struct_named, struct_newtype, struct_tuple, struct_unit)
)]
struct NamespaceAttrs {
    ident: syn::Ident,
    name: Option<String>,
}

pub fn expand(input: TokenStream) -> TokenStream {
    let derive_input: DeriveInput = match syn::parse2(input) {
        Ok(input) => input,
        Err(err) => return err.into_compile_error(),
    };
    let NamespaceAttrs { name, ident } = match NamespaceAttrs::from_derive_input(&derive_input) {
        Ok(attrs) => attrs,
        Err(err) => return err.write_errors(),
    };

    let name = name.unwrap_or_else(|| ident.to_string());

    quote! {
        impl #zod_core::Namespace for #ident {
            const NAME: &'static str = #name;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils::TokenStreamExt;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn default_name() {
        let input = quote! {
            struct MyNs;
        };

        let expected = quote! {
            impl #zod_core::Namespace for MyNs {
                const NAME: &'static str = "MyNs";
            }

        };

        assert_eq!(
            expand(input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        );
    }

    #[test]
    fn custom_name() {
        let input = quote! {
            #[zod(name = "Custom_Name")]
            struct MyNs;
        };

        let expected = quote! {
            impl #zod_core::Namespace for MyNs {
                const NAME: &'static str = "Custom_Name";
            }

        };

        assert_eq!(
            expand(input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        );
    }
}
