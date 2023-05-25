use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{quote, quote_spanned, ToTokens};
use syn::spanned::Spanned;
use syn::{parse_quote, DataStruct, DeriveInput, Generics};

use crate::{types::Role, utils::crate_name};

fn qualify_ty(ty: &syn::Type, trait_path: syn::Path) -> TokenStream2 {
    let span = ty.span();
    quote_spanned!(span => < #ty as #trait_path>)
}

fn impl_zod_object(fields: &syn::FieldsNamed) -> TokenStream2 {
    let fields = fields.named.iter().map(|f| {
        let ident = f.ident.as_ref().expect("named fields");
        let name = ident.to_string();
        let optional = false; //todo

        let ty = &f.ty;

        let qualified_ty = qualify_ty(ty, parse_quote!(#crate_name::IoType));

        quote_spanned! {
            ty.span() =>
            #crate_name::types::ZodNamedField {
                optional: #optional,
                name: #name,
                value: #crate_name::types::ZodType::from(#qualified_ty::get_ref())
            }
        }
    });

    quote!(#crate_name::types::ZodObject {
        fields: vec![#(#fields),*]
    }.into())
}

fn impl_zod_tuple(fields: &syn::FieldsUnnamed) -> TokenStream2 {
    let fields = fields.unnamed.iter().map(|f| {
        let ty = &f.ty;

        let qualified_ty = qualify_ty(ty, parse_quote!(#crate_name::IoType));

        quote_spanned! {
            ty.span() =>
            #crate_name::types::ZodType::from(#qualified_ty::get_ref())
        }
    });

    quote!(#crate_name::types::ZodTuple {
        fields: vec![#(#fields),*]
    }.into())
}

struct StructImpl {
    ident: Ident,
    generics: Generics,
    data: DataStruct,
}

impl ToTokens for StructImpl {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ns = "Ns"; //todo
        let role = Role::Io; //todo
        let ident = &self.ident;
        let name = self.ident.to_string();

        let custom_suffix = quote!(None);

        let make_export_stmts = |ty: &syn::Type| {
            let qualified_ty = qualify_ty(ty, parse_quote!(#crate_name::IoType));
            quote_spanned!(ty.span() => #qualified_ty::visit_exports(set))
        };

        let exports: Vec<_> = match &self.data.fields {
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .map(|f| make_export_stmts(&f.ty))
                .collect(),

            syn::Fields::Unnamed(fields) => fields
                .unnamed
                .iter()
                .map(|f| make_export_stmts(&f.ty))
                .collect(),

            syn::Fields::Unit => todo!(),
        };

        let inner = match &self.data.fields {
            syn::Fields::Named(fields) => impl_zod_object(fields),
            syn::Fields::Unnamed(fields) => impl_zod_tuple(fields),
            syn::Fields::Unit => todo!(),
        };

        let arg_names = self
            .generics
            .params
            .iter()
            .map(|p| match p {
                syn::GenericParam::Lifetime(_) => todo!(),
                syn::GenericParam::Type(param) => param.ident.to_string(),
                syn::GenericParam::Const(_) => todo!(),
            })
            .collect::<Vec<_>>();

        tokens.extend(quote!(impl #crate_name::IoType for #ident {
            fn get_ref() -> #crate_name::types::ZodType {
                #crate_name::Reference {
                    name: ::std::string::String::from(#name),
                    ns: ::std::string::String::from(#ns),
                    role: #role,
                    args: vec![#(#arg_names),*]
                }.into()
            }

            fn visit_exports(set: &mut ::std::collections::HashSet<#crate_name::types::ZodExport>) {
                let export = #crate_name::types::ZodExport {
                    ns: ::std::string::String::from(#ns),
                    name: ::std::string::String::from(#name),
                    context: #role,
                    args: &[#(#arg_names),*],
                    value: #crate_name::types::ZodType {
                        optional: false,
                        custom_suffix: #custom_suffix,
                        inner: #inner
                    }
                };

                set.insert(export);
                #(#exports;)*
            }

        }))
    }
}

pub fn impl_zod(input: TokenStream2) -> TokenStream2 {
    let parsed: DeriveInput = match syn::parse2(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let ident = parsed.ident;
    let generics = parsed.generics;

    match parsed.data {
        syn::Data::Struct(data) => {
            let it = StructImpl {
                ident,
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
    use crate::{test_utils::TokenStreamExt, utils::crate_name};
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
            impl #crate_name::IoType for Test {
                fn get_ref() -> #crate_name::types::ZodType {
                    #crate_name::Reference {
                        name: ::std::string::String::from("Test"),
                        ns: ::std::string::String::from("Ns"),
                        role: #role,
                        args: vec![]
                    }.into()
                }

                fn visit_exports(set: &mut HashSet<#crate_name::types::ZodExport>) {
                    let export = #crate_name::types::ZodExport {
                        ns: ::std::string::String::from("Ns"),
                        name: ::std::string::String::from("Test"),
                        context: #role,
                        args: &[],
                        value: #crate_name::types::ZodType {
                            optional: false,
                            custom_suffix: None,
                            inner: #crate_name::types::ZodObject {
                                fields: vec![
                                    #crate_name::types::ZodNamedField {
                                        name: "inner",
                                        optional: false,
                                        value: #crate_name::types::ZodType::from(<String as #crate_name::IoType>::get_ref())
                                    },
                                    #crate_name::types::ZodNamedField {
                                        name: "inner2",
                                        optional: false,
                                        value: #crate_name::types::ZodType::from(<usize as #crate_name::IoType>::get_ref())
                                    }
                                ]
                            }.into()
                        }
                    };

                    set.insert(export);
                    <String as #crate_name::IoType>::visit_exports(set);
                    <usize as #crate_name::IoType>::visit_exports(set);
                }
            }
        };

        assert_eq!(
            impl_zod(input).to_formatted_string(),
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
            impl #crate_name::IoType for Test {
                fn get_ref() -> #crate_name::types::ZodType {
                    #crate_name::Reference {
                        name: ::std::string::String::from("Test"),
                        ns: ::std::string::String::from("Ns"),
                        role: #role,
                        args: vec![]
                    }.into()
                }

                fn visit_exports(set: &mut ::std::collections::HashSet<#crate_name::types::ZodExport>) {
                        let export = #crate_name::types::ZodExport {
                            ns: ::std::string::String::from("Ns"),
                            name: ::std::string::String::from("Test"),
                            context: #role,
                            args: &[],
                            value: #crate_name::types::ZodType {
                                optional: false,
                                custom_suffix: None,
                                inner: #crate_name::types::ZodTuple {
                                    fields: vec![
                                        #crate_name::types::ZodType::from(<String as #crate_name::IoType>::get_ref()),
                                        #crate_name::types::ZodType::from(<usize as #crate_name::IoType>::get_ref())
                                    ]
                                }.into()
                            }
                        };

                    set.insert(export);
                    <String as #crate_name::IoType>::visit_exports(set);
                    <usize as #crate_name::IoType>::visit_exports(set);
                }
            }
        };

        assert_eq!(
            impl_zod(input).to_formatted_string(),
            expected.to_formatted_string()
        )
    }
}
