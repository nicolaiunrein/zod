mod struct_impl;
use crate::derive_internals::struct_impl::StructImpl;
use crate::types::Role;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::DeriveInput;

fn qualify_ty(ty: &syn::Type, trait_path: syn::Path) -> TokenStream2 {
    let span = ty.span();
    quote_spanned!(span => < #ty as #trait_path>)
}

pub fn impl_zod(role: Role, input: TokenStream2) -> TokenStream2 {
    let parsed: DeriveInput = match syn::parse2(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let ident = parsed.ident;
    let generics = parsed.generics;
    let ns = String::from("Ns"); //todo

    match parsed.data {
        syn::Data::Struct(data) => {
            let it = StructImpl {
                ident,
                role,
                ns,
                generics,
                data,
            };

            quote!(#it)
        }
        syn::Data::Enum(_) => todo!(),
        syn::Data::Union(_) => todo!(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{test_utils::TokenStreamExt, types::Role, utils::zod_core};
    use pretty_assertions::assert_eq;

    #[test]
    fn zod_named_struct_ok() {
        let input = quote! {
            struct Test {
                inner: String,
                inner2: usize
            }
        };

        let role = Role::Io;

        let expected = quote! {
            impl #zod_core::IoType for Test {
                fn get_ref() -> #zod_core::types::ZodType {
                    #zod_core::Reference {
                        name: ::std::string::String::from("Test"),
                        ns: ::std::string::String::from("Ns"),
                        role: #role,
                        args: vec![]
                    }.into()
                }

                fn visit_exports(set: &mut ::std::collections::HashSet<#zod_core::types::ZodExport>) {
                    let export = #zod_core::types::ZodExport {
                        ns: ::std::string::String::from("Ns"),
                        name: ::std::string::String::from("Test"),
                        context: #role,
                        args: &[],
                        value: #zod_core::types::ZodType {
                            optional: false,
                            custom_suffix: None,
                            inner: #zod_core::types::ZodObject {
                                fields: vec![
                                    #zod_core::types::ZodNamedField {
                                        optional: false,
                                        name: "inner",
                                        value: #zod_core::types::ZodType::from(<String as #zod_core::IoType>::get_ref())
                                    },
                                    #zod_core::types::ZodNamedField {
                                        optional: false,
                                        name: "inner2",
                                        value: #zod_core::types::ZodType::from(<usize as #zod_core::IoType>::get_ref())
                                    }
                                ]
                            }.into()
                        }
                    };

                    set.insert(export);
                    <String as #zod_core::IoType>::visit_exports(set);
                    <usize as #zod_core::IoType>::visit_exports(set);
                }
            }
        };

        assert_eq!(
            impl_zod(Role::Io, input).to_formatted_string(),
            expected.to_formatted_string()
        )
    }

    #[test]
    fn zod_tuple_struct_ok() {
        let input = quote! {
            struct Test(String, usize);
        };

        let role = Role::Io;

        let expected = quote! {
            impl #zod_core::IoType for Test {
                fn get_ref() -> #zod_core::types::ZodType {
                    #zod_core::Reference {
                        name: ::std::string::String::from("Test"),
                        ns: ::std::string::String::from("Ns"),
                        role: #role,
                        args: vec![]
                    }.into()
                }

                fn visit_exports(set: &mut ::std::collections::HashSet<#zod_core::types::ZodExport>) {
                        let export = #zod_core::types::ZodExport {
                            ns: ::std::string::String::from("Ns"),
                            name: ::std::string::String::from("Test"),
                            context: #role,
                            args: &[],
                            value: #zod_core::types::ZodType {
                                optional: false,
                                custom_suffix: None,
                                inner: #zod_core::types::ZodTuple {
                                    fields: vec![
                                        #zod_core::types::ZodType::from(<String as #zod_core::IoType>::get_ref()),
                                        #zod_core::types::ZodType::from(<usize as #zod_core::IoType>::get_ref())
                                    ]
                                }.into()
                            }
                        };

                    set.insert(export);
                    <String as #zod_core::IoType>::visit_exports(set);
                    <usize as #zod_core::IoType>::visit_exports(set);
                }
            }
        };

        assert_eq!(
            impl_zod(Role::Io, input).to_formatted_string(),
            expected.to_formatted_string()
        )
    }
}
