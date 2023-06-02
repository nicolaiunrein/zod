mod ast;
mod attrs;
mod custom_suffix;
mod r#enum;
mod fields;
mod generics;
mod r#struct;

use self::ast::Ast;
use crate::utils::zod_core;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::DeriveInput;

#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum Derive {
    Input,
    Output,
}

impl ToTokens for Derive {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Input => tokens.extend(quote!(#zod_core::Kind::Input)),
            Self::Output => tokens.extend(quote!(#zod_core::Kind::Output)),
        }
    }
}

/// convert input into the generated code providing a `Derive`.
pub fn expand(derive: Derive, input: TokenStream2) -> TokenStream2 {
    let derive_input: DeriveInput = match syn::parse2(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error();
        }
    };

    let ast = match Ast::new(derive, derive_input) {
        Ok(attrs) => attrs,
        Err(err) => return err,
    };

    ast.into_token_stream()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        derive_internals::zod::{
            custom_suffix::CustomSuffix,
            r#enum::EnumImpl,
            r#struct::{ZodObjectImpl, ZodTupleImpl},
        },
        test_utils::TokenStreamExt,
    };
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn expand_zod_for_struct_with_named_fields_ok() {
        let derive = Derive::Input;
        let input = quote! {
            #[zod(namespace = "Ns")]
            struct Test {
                inner_string: String,
                inner_u8: u8
            }
        };

        let inner = ZodObjectImpl::new(
            derive,
            &parse_quote!({ inner_string: String, inner_u8: u8 }),
        );

        let custom_suffix = CustomSuffix { inner: None };

        let expected = quote! {
            impl #zod_core::Type<#derive> for Test {
                type Ns = Ns;
                const NAME: &'static str = "Test";
                const INLINE: bool = false;

                fn value() -> #zod_core::z::ZodType<#derive> {
                    #zod_core::z::ZodType {
                        optional: false,
                        custom_suffix: #custom_suffix,
                        inner: #inner.into()
                    }
                }

                fn args() -> ::std::vec::Vec<#zod_core::GenericArgument<#derive>> {
                    let mut v = ::std::vec::Vec::new();
                    v
                }

                fn visit_dependencies(visitor: &mut #zod_core::DependencyVisitor<#zod_core::Kind::Input>) {}
            }

            impl Ns {
                #[allow(dead_code)]
                #[allow(non_upper_case_globals)]
                const __ZOD_PRIVATE_INPUT___Test: () = {};
            }

        };

        assert_eq!(
            expand(Derive::Input, input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        )
    }

    #[test]
    fn expand_zod_for_struct_with_tuple_fields_ok() {
        let derive = Derive::Input;
        let input = quote! {
            #[zod(namespace = "Ns")]
            struct Test(String, u8);
        };

        let custom_suffix = CustomSuffix { inner: None };

        let inner = ZodTupleImpl::new(derive, &parse_quote!((String, u8)));

        let expected = quote! {
            impl #zod_core::Type<#derive> for Test {
                type Ns = Ns;
                const NAME: &'static str = "Test";
                const INLINE: bool = false;

                fn value() -> #zod_core::z::ZodType<#derive> {
                    #zod_core::z::ZodType {
                        optional: false,
                        custom_suffix: #custom_suffix,
                        inner: #inner.into()
                    }
                }

                fn args() -> ::std::vec::Vec<#zod_core::GenericArgument<#derive>> {
                    let mut v = ::std::vec::Vec::new();
                    v
                }

                fn visit_dependencies(visitor: &mut #zod_core::DependencyVisitor<#zod_core::Kind::Input>) {}
            }

            impl Ns {
                #[allow(dead_code)]
                #[allow(non_upper_case_globals)]
                const __ZOD_PRIVATE_INPUT___Test: () = {};
            }

        };

        assert_eq!(
            expand(Derive::Input, input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        )
    }

    #[test]
    fn impl_zod_for_enum() {
        let derive = Derive::Input;
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

        let inner = EnumImpl::new(
            derive,
            Default::default(),
            vec![
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
        );

        let custom_suffix = CustomSuffix { inner: None };

        let expected = quote! {
            impl #zod_core::Type<#derive> for Test {
                type Ns = Ns;
                const NAME: &'static str = "Test";
                const INLINE: bool = false;

                fn value() -> #zod_core::z::ZodType<#derive> {
                    #zod_core::z::ZodType {
                        optional: false,
                        custom_suffix: #custom_suffix,
                        inner: #inner.into()
                    }
                }

                fn args() -> ::std::vec::Vec<#zod_core::GenericArgument<#derive>> {
                    let mut v = ::std::vec::Vec::new();
                    v
                }

                fn visit_dependencies(visitor: &mut #zod_core::DependencyVisitor<#zod_core::Kind::Input>) {}
            }

            impl Ns {
                #[allow(dead_code)]
                #[allow(non_upper_case_globals)]
                const __ZOD_PRIVATE_INPUT___Test: () = {};
            }

        };

        assert_eq!(
            expand(Derive::Input, input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        );
    }
}
