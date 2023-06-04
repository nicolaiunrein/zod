mod ast;
mod attrs;
mod custom_suffix;
mod data;
mod r#enum;
mod fields;
mod generics;
mod r#struct;
mod variant;

use self::ast::Ast;
use crate::utils::zod_core;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use serde_derive_internals::Derive as SerdeDerive;
use syn::DeriveInput;

#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum Derive {
    Input,
    Output,
}

impl From<Derive> for SerdeDerive {
    fn from(value: Derive) -> Self {
        match value {
            Derive::Input => SerdeDerive::Deserialize,
            Derive::Output => SerdeDerive::Serialize,
        }
    }
}

impl ToTokens for Derive {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Input => tokens.extend(quote!(#zod_core::Kind::Input)),
            Self::Output => tokens.extend(quote!(#zod_core::Kind::Output)),
        }
    }
}

/// convert input into the generated code providing a `Derive`.
pub fn expand(derive: Derive, input: TokenStream) -> TokenStream {
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
            fields::{FieldValue, ZodNamedFieldImpl},
            r#struct::ZodObjectImpl,
        },
        test_utils::TokenStreamExt,
    };
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn expand_ok() {
        let derive = Derive::Input;
        let input = quote! {
            #[zod(namespace = "Ns")]
            struct Test {
                inner_string: String,
                inner_u8: u8
            }
        };

        let inner = ZodObjectImpl {
            fields: vec![
                ZodNamedFieldImpl {
                    name: String::from("inner_string"),
                    optional: false,
                    derive,
                    value: FieldValue::Type(parse_quote!(String)),
                },
                ZodNamedFieldImpl {
                    name: String::from("inner_u8"),
                    optional: false,
                    derive,
                    value: FieldValue::Type(parse_quote!(u8)),
                },
            ],
        };

        let custom_suffix = CustomSuffix { inner: None };

        let expected = quote! {
            const _: () = {
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
        };

        assert_eq!(
            expand(Derive::Input, input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        )
    }
}
