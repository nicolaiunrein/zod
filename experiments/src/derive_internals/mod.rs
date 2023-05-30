mod r#struct;
use crate::utils::zod_core;
use crate::Kind;
use darling::FromDeriveInput;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use r#struct::StructImpl;
use syn::DeriveInput;

impl ToTokens for Kind::Input {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(quote!(#zod_core::Kind::Input))
    }
}

impl ToTokens for Kind::Output {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        tokens.extend(quote!(#zod_core::Kind::Output))
    }
}

#[derive(FromDeriveInput)]
#[darling(
    attributes(zod),
    forward_attrs(allow, doc, cfg),
    supports(struct_named, struct_newtype, struct_tuple, enum_any)
)]

struct ZodOptions {
    pub namespace: syn::Path,
    pub custom_suffix: Option<String>,
}

/// convert input into the generated code providing a role.
pub fn impl_zod<Io>(role: Io, input: TokenStream2) -> TokenStream2
where
    Io: ToTokens + Copy,
{
    let derive_input: DeriveInput = match syn::parse2(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error();
        }
    };

    let attrs: ZodOptions = match ZodOptions::from_derive_input(&derive_input) {
        Ok(attrs) => attrs,
        Err(err) => {
            return err.write_errors();
        }
    };

    let ident = derive_input.ident;
    let generics = derive_input.generics;

    match derive_input.data {
        syn::Data::Struct(data) => StructImpl {
            role,
            ident,
            ns: attrs.namespace,
            custom_suffix: attrs.custom_suffix,
            generics,
            fields: data.fields,
        }
        .into_token_stream(),

        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{test_utils::TokenStreamExt, Kind};
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn impl_zod_for_struct_with_named_fields_ok() {
        let input = quote! {
            #[zod(namespace = "Ns")]
            struct Test {
                inner_string: String,
                inner_u8: u8
            }
        };

        let expected = StructImpl {
            ident: parse_quote!(Test),
            generics: Default::default(),
            fields: syn::Fields::Named(parse_quote!({ inner_string: String, inner_u8: u8 })),
            role: Kind::Input,
            ns: parse_quote!(Ns),
            custom_suffix: None,
        }
        .into_token_stream();

        assert_eq!(
            impl_zod(Kind::Input, input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        )
    }

    #[test]
    fn impl_zod_for_struct_with_tuple_fields_ok() {
        let input = quote! {
            #[zod(namespace = "Ns")]
            struct Test(String, u8);
        };

        let expected = StructImpl {
            ident: parse_quote!(Test),
            generics: Default::default(),
            fields: syn::Fields::Unnamed(parse_quote!((String, u8))),
            role: Kind::Input,
            ns: parse_quote!(Ns),
            custom_suffix: None,
        }
        .into_token_stream();

        assert_eq!(
            impl_zod(Kind::Input, input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        )
    }

    #[ignore]
    #[test]
    fn impl_zod_for_enum() {
        let input = quote! {
            #[zod(namespace = "Ns")]
            enum Test {
                Unit,
                Tuple1(String),
                Tuple2(String, u8),
                Struct0 {},
                Struct1 {
                    inner: String,
                },
                Struct2 {
                    inner_string: String,
                    inner_u8: u8,
                }
            }
        };

        let expected: TokenStream2 = todo!();

        assert_eq!(
            impl_zod(Kind::Input, input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        )
    }
}
