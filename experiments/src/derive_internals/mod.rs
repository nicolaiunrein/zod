mod r#enum;
mod fields;
mod r#struct;

use crate::utils::zod_core;
use crate::Kind;
use darling::FromDeriveInput;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use r#enum::EnumImpl;
use r#struct::StructImpl;
use serde_derive_internals::attr::TagType as SerdeTagType;
use syn::DeriveInput;

use self::r#enum::TagType;

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

impl From<&SerdeTagType> for TagType {
    fn from(value: &SerdeTagType) -> Self {
        match value {
            SerdeTagType::External => TagType::Externally,
            SerdeTagType::Internal { tag } => TagType::Internally {
                tag: tag.to_owned(),
            },
            SerdeTagType::Adjacent { tag, content } => TagType::Adjacently {
                tag: tag.to_owned(),
                content: content.to_owned(),
            },
            SerdeTagType::None => TagType::Untagged,
        }
    }
}

/// convert input into the generated code providing a kind.
pub fn impl_zod<Io>(kind: Io, input: TokenStream2) -> TokenStream2
where
    Io: ToTokens + Copy,
{
    let derive_input: DeriveInput = match syn::parse2(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error();
        }
    };

    let cx = serde_derive_internals::Ctxt::new();
    let serde_attrs = serde_derive_internals::attr::Container::from_ast(&cx, &derive_input);
    cx.check().unwrap();

    let attrs: ZodOptions = match ZodOptions::from_derive_input(&derive_input) {
        Ok(attrs) => attrs,
        Err(err) => {
            return err.write_errors();
        }
    };

    let ident = derive_input.ident;
    let generics = derive_input.generics;

    let inner = match derive_input.data {
        syn::Data::Struct(data) => StructImpl {
            kind,
            fields: data.fields,
        }
        .into_token_stream(),

        syn::Data::Enum(data) => EnumImpl {
            tag: serde_attrs.tag().into(),
            kind,
            variants: data.variants.into_iter().collect(),
        }
        .into_token_stream(),

        syn::Data::Union(_) => todo!("todo... not supported"),
    };

    let ns = attrs.namespace;
    let name = ident.to_string();

    let arg_idents = generics
        .params
        .iter()
        .map(|p| match p {
            syn::GenericParam::Lifetime(_) => todo!(),
            syn::GenericParam::Type(param) => &param.ident,
            syn::GenericParam::Const(_) => todo!(),
        })
        .collect::<Vec<_>>();

    let custom_suffix = match attrs.custom_suffix {
        Some(suffix) => quote!(::std::option::Option::Some(::std::string::String::from(
            #suffix
        ))),
        None => quote!(None),
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics #zod_core::Type<#kind> for #ident #ty_generics #where_clause {
            type Ns = #ns;
            const NAME: &'static str = #name;

            fn value() -> #zod_core::types::ZodType<#kind> {
                #zod_core::types::ZodType {
                    optional: false,
                    custom_suffix: #custom_suffix,
                    inner: #inner.into()
                }
            }

            fn args() -> #zod_core::GenericArguments<#kind> {
                #zod_core::make_args!(#(#arg_idents),*)
            }
        }
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
        let kind = Kind::Input;
        let input = quote! {
            #[zod(namespace = "Ns")]
            struct Test {
                inner_string: String,
                inner_u8: u8
            }
        };

        let inner = StructImpl {
            fields: syn::Fields::Named(parse_quote!({ inner_string: String, inner_u8: u8 })),
            kind,
        };

        let custom_suffix = quote!(None);

        let expected = quote! {
            impl #zod_core::Type<#kind> for Test {
                type Ns = Ns;
                const NAME: &'static str = "Test";

                fn value() -> #zod_core::types::ZodType<#kind> {
                    #zod_core::types::ZodType {
                        optional: false,
                        custom_suffix: #custom_suffix,
                        inner: #inner.into()
                    }
                }

                fn args() -> #zod_core::GenericArguments<#kind> {
                    #zod_core::make_args!()
                }
            }

        };

        assert_eq!(
            impl_zod(Kind::Input, input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        )
    }

    #[test]
    fn impl_zod_for_struct_with_tuple_fields_ok() {
        let kind = Kind::Input;
        let input = quote! {
            #[zod(namespace = "Ns")]
            struct Test(String, u8);
        };

        let custom_suffix = quote!(None);

        let inner = StructImpl {
            fields: syn::Fields::Unnamed(parse_quote!((String, u8))),
            kind,
        };

        let expected = quote! {
            impl #zod_core::Type<#kind> for Test {
                type Ns = Ns;
                const NAME: &'static str = "Test";

                fn value() -> #zod_core::types::ZodType<#kind> {
                    #zod_core::types::ZodType {
                        optional: false,
                        custom_suffix: #custom_suffix,
                        inner: #inner.into()
                    }
                }

                fn args() -> #zod_core::GenericArguments<#kind> {
                    #zod_core::make_args!()
                }
            }

        };

        assert_eq!(
            impl_zod(Kind::Input, input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        )
    }

    #[test]
    fn impl_zod_for_enum() {
        let kind = Kind::Input;
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

        let inner = EnumImpl {
            tag: Default::default(),
            kind,
            variants: vec![
                parse_quote!(Unit),
                parse_quote!(Tuple1(String)),
                parse_quote!(Tuple2(String, u8)),
                parse_quote!(Struct0 {}),
                parse_quote!(Struct1 { inner: String }),
                parse_quote!(Struct2 {
                    inner_string: String,
                    inner_u8: u8
                }),
            ],
        };

        let custom_suffix = quote!(None);

        let expected = quote! {
            impl #zod_core::Type<#kind> for Test {
                type Ns = Ns;
                const NAME: &'static str = "Test";

                fn value() -> #zod_core::types::ZodType<#kind> {
                    #zod_core::types::ZodType {
                        optional: false,
                        custom_suffix: #custom_suffix,
                        inner: #inner.into()
                    }
                }

                fn args() -> #zod_core::GenericArguments<#kind> {
                    #zod_core::make_args!()
                }
            }

        };

        assert_eq!(
            impl_zod(Kind::Input, input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        )
    }
}
