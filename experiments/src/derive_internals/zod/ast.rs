use super::custom_suffix::CustomSuffix;
use super::generics::{needs_inline, replace_generics, GenericsExt};
use super::r#enum::{EnumImpl, TagType};
use super::r#struct::StructImpl;
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
    // pub transparent: bool,
    // pub default
    // pub type_from: Option<syn::Type>,
    // pub type_try_from: Option<syn::Type>,
    // pub type_into: Option<syn::Type>,
}

impl Ast {
    pub fn new(derive: Derive, mut derive_input: DeriveInput) -> Result<Self, TokenStream2> {
        let cx = serde_derive_internals::Ctxt::new();
        let serde_attrs = serde_derive_internals::attr::Container::from_ast(&cx, &derive_input);
        cx.check().unwrap();

        let zod_attrs: ZodAttrs = match ZodAttrs::from_derive_input(&derive_input) {
            Ok(attrs) => attrs,
            Err(err) => return Err(err.write_errors()),
        };

        derive_input.generics.update_where_clause(derive);

        Ok(Self {
            derive,
            ident: derive_input.ident.clone(),
            data: Data::new(derive, derive_input.clone(), serde_attrs.tag().into()),
            generics: derive_input.generics,
            namespace: zod_attrs.namespace,
            custom_suffix: CustomSuffix {
                inner: zod_attrs.custom_suffix,
            },
            name: match derive {
                Derive::Input => serde_attrs.name().deserialize_name(),
                Derive::Output => serde_attrs.name().serialize_name(),
            },
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

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        tokens.extend(quote! {
            impl #impl_generics #zod_core::Type<#derive> for #ident #ty_generics #where_clause {
                type Ns = #ns;
                const NAME: &'static str = #name;
                const INLINE: bool = #inline;

                fn value() -> #zod_core::z::ZodType<#derive> {
                    #zod_core::z::ZodType {
                        optional: false,
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
            }

            impl #ns {
                #[allow(dead_code)]
                #[allow(non_upper_case_globals)]
                const #unique_ident: () = {};
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub(super) enum Data {
    Struct(bool, StructImpl),
    Enum(bool, EnumImpl),
}

impl ToTokens for Data {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Data::Struct(_, inner) => inner.to_tokens(tokens),
            Data::Enum(_, inner) => inner.to_tokens(tokens),
        }
    }
}

impl Data {
    fn new(derive: Derive, input: syn::DeriveInput, tag: TagType) -> Self {
        match input.data {
            syn::Data::Struct(mut data) => {
                let inline = data
                    .fields
                    .iter()
                    .any(|f| needs_inline(&f.ty, &input.generics));

                if !inline {
                    for f in data.fields.iter_mut() {
                        replace_generics(&mut f.ty, &input.generics);
                    }
                }

                Self::Struct(
                    inline,
                    StructImpl {
                        derive,
                        fields: data.fields,
                    },
                )
            }

            syn::Data::Enum(data) => {
                let inline = data.variants.iter().any(|v| {
                    v.fields
                        .iter()
                        .any(|f| needs_inline(&f.ty, &input.generics))
                });

                let variants = if !inline {
                    data.variants
                        .into_iter()
                        .map(|mut v| {
                            for f in v.fields.iter_mut() {
                                replace_generics(&mut f.ty, &input.generics);
                            }
                            v
                        })
                        .collect::<Vec<_>>()
                } else {
                    data.variants.into_iter().collect::<Vec<_>>()
                };

                Self::Enum(inline, EnumImpl::new(derive, tag, variants))
            }

            syn::Data::Union(_) => todo!("todo... not supported"),
        }
    }

    fn inline(&self) -> bool {
        match self {
            Data::Struct(inline, _) => *inline,
            Data::Enum(inline, _) => *inline,
        }
    }
}

#[cfg(test)]
mod test {
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

        let data = Data::new(Derive::Input, input, Default::default());

        assert_eq!(
            data,
            Data::Struct(
                false,
                StructImpl::new(
                    Derive::Input,
                    syn::Fields::Named(
                        parse_quote!({inner: Other<#zod_core::typed_str::TypedStr<'T', #zod_core::typed_str::End>>})
                    )
                )
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

        let data = Data::new(Derive::Input, input, Default::default());

        assert_eq!(
            data,
            Data::Struct(
                true,
                StructImpl::new(
                    Derive::Input,
                    syn::Fields::Named(parse_quote!({inner: Other<T>}))
                )
            )
        )
    }
}
