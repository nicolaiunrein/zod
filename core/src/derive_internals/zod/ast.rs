use super::attrs::get_rustdoc;
use super::custom_suffix::CustomSuffix;
use super::data::Data;
use super::generics::GenericsExt;
use super::Derive;
use super::{attrs::ZodAttrs, custom_suffix};
use crate::utils::zod_core;
use crate::Kind;
use darling::FromDeriveInput;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::DeriveInput;

pub(super) struct Ast {
    derive: Derive,
    pub ident: syn::Ident,
    pub data: Data,
    pub generics: syn::Generics,
    pub namespace: syn::Path,
    pub custom_suffix: custom_suffix::CustomSuffix,
    pub name: String,
    pub optional: bool,
    pub docs: Option<String>,
    // pub transparent: bool,
    // pub type_from: Option<syn::Type>,
    // pub type_try_from: Option<syn::Type>,
    // pub type_into: Option<syn::Type>,
}

impl Ast {
    pub fn new(derive: Derive, mut derive_input: DeriveInput) -> Result<Self, TokenStream2> {
        let cx = serde_derive_internals::Ctxt::new();
        let input_clone = derive_input.clone();
        let serde_ast =
            serde_derive_internals::ast::Container::from_ast(&cx, &input_clone, derive.into())
                .unwrap();

        cx.check().unwrap();

        let zod_attrs: ZodAttrs = match ZodAttrs::from_derive_input(&derive_input) {
            Ok(attrs) => attrs,
            Err(err) => return Err(err.write_errors()),
        };

        derive_input.generics.update_where_clause(derive);

        let name = match derive {
            Derive::Input => serde_ast.attrs.name().deserialize_name(),
            Derive::Output => serde_ast.attrs.name().serialize_name(),
        };

        let mut custom_suffix = CustomSuffix {
            inner: zod_attrs.custom_suffix,
        };

        if serde_ast.attrs.deny_unknown_fields() {
            custom_suffix.add(".strict()");
        }

        Ok(Self {
            derive,
            ident: derive_input.ident.clone(),
            docs: get_rustdoc(&derive_input.attrs),
            optional: !serde_ast.attrs.default().is_none(),
            data: Data::new(derive, serde_ast),
            generics: derive_input.generics,
            namespace: zod_attrs.namespace,
            custom_suffix,
            name,
        })
    }

    fn generic_arguments(&self) -> Vec<TokenStream2> {
        self.generics
            .idents()
            .iter()
            .map(|ident| {
                let name = ident.to_string();

                quote_spanned! {
                    ident.span() =>
                    #zod_core::GenericArgument::new::<#ident>(#name)
                }
            })
            .collect()
    }

    fn unique_ident(&self) -> syn::Ident {
        let name = &self.name;
        match self.derive {
            Derive::Input => {
                crate::utils::make_unique_name::<Kind::Input>(&quote::format_ident!("{name}"))
            }
            Derive::Output => {
                crate::utils::make_unique_name::<Kind::Output>(&quote::format_ident!("{name}"))
            }
        }
    }
}

impl ToTokens for Ast {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let ident = &self.ident;
        let ns = &self.namespace;
        let name = &self.name;
        let custom_suffix = &self.custom_suffix;
        let inline = self.data.inline();
        let inner = &self.data;
        let generic_arguments = self.generic_arguments();
        let unique_ident = self.unique_ident();
        let derive = self.derive;
        let optional = self.optional;

        let docs = if let Some(ref docs) = self.docs {
            quote! {
                fn docs() -> ::std::option::Option<::std::string::String> {
                    Some(::std::string::String::from(#docs))
                }
            }
        } else {
            quote!()
        };

        let generic_impls = self.generics.idents().into_iter().map(|ident| {
            let name = ident.to_string();
            let ident = super::generics::make_generic_struct_ident(ident);
            quote! {
                struct #ident;

                impl #zod_core::Generic for #ident {
                    type Ns = #ns;
                    const VALUE: &'static str = #name;
                }
            }
        });

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        tokens.extend(quote! {
            const _: () = {
                #(#generic_impls)*



            impl #impl_generics #zod_core::Type<#derive> for #ident #ty_generics #where_clause {
                type Ns = #ns;
                const NAME: &'static str = #name;
                const INLINE: bool = #inline;

                fn value() -> #zod_core::z::ZodType<#derive> {
                    #zod_core::z::ZodType {
                        optional: #optional,
                        custom_suffix: #custom_suffix,
                        inner: #inner.into()
                    }
                }

                fn args() -> ::std::vec::Vec<#zod_core::GenericArgument<#derive>> {
                    let mut v = ::std::vec::Vec::new();
                    #(v.push(#generic_arguments);)*
                    v
                }

                fn visit_dependencies(visitor: &mut #zod_core::DependencyVisitor<#derive>) {
                    // TODO
                }

                #docs
            }

            impl #ns {
                #[allow(dead_code)]
                #[allow(non_upper_case_globals)]
                const #unique_ident: () = {};
            }
            };
        })
    }
}

#[cfg(test)]
mod test {
    use crate::derive_internals::zod::{
        fields::{FieldValue, ZodNamedFieldImpl},
        r#struct::ZodObjectImpl,
    };

    use super::*;
    use pretty_assertions::assert_eq;
    use syn::parse_quote;

    #[test]
    fn non_inline_generics_ok() {
        let input = parse_quote! {
            struct X<T> {
                inner: Other<T>
            }
        };

        let cx = serde_derive_internals::Ctxt::new();
        let input = serde_derive_internals::ast::Container::from_ast(
            &cx,
            &input,
            serde_derive_internals::Derive::Serialize,
        )
        .unwrap();
        cx.check().unwrap();

        let data = Data::new(Derive::Input, input);

        assert_eq!(
            data,
            Data::Struct(
                false,
                ZodObjectImpl {
                    fields: vec![ZodNamedFieldImpl {
                        name: String::from("inner"),
                        optional: false,
                        derive: Derive::Input,
                        value: FieldValue::Type(
                            parse_quote!(Other<#zod_core::GenericPlaceholder<__GENERIC_T>>),
                        )
                    }]
                }
            )
        )
    }

    #[test]
    fn inline_generics_ok() {
        let input = parse_quote! {
            struct X<T: SomeTrait> {
                inner: Other<T>
            }
        };

        let cx = serde_derive_internals::Ctxt::new();
        let input = serde_derive_internals::ast::Container::from_ast(
            &cx,
            &input,
            serde_derive_internals::Derive::Serialize,
        )
        .unwrap();
        cx.check().unwrap();

        let data = Data::new(Derive::Input, input);

        assert_eq!(
            data,
            Data::Struct(
                true,
                ZodObjectImpl {
                    fields: vec![ZodNamedFieldImpl {
                        name: String::from("inner"),
                        optional: false,
                        derive: Derive::Input,
                        value: FieldValue::Type(parse_quote!(Other<T>))
                    }]
                }
            )
        )
    }
}
