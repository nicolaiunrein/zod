use crate::config::ContainerConfig;
use crate::error::Error;
use crate::field::Field;
use crate::field::FilteredFields;
use crate::r#enum::Enum;
use crate::r#struct::Struct;
use crate::utils::get_zod;
use darling::FromDeriveInput;
use darling::ToTokens;
use proc_macro2::TokenStream;
use quote::quote;
use serde_derive_internals::ast::Data;
use syn::Type;
use syn::{Attribute, Generics};

#[derive(FromDeriveInput)]
#[darling(
    attributes(zod),
    forward_attrs(allow, doc, cfg),
    supports(struct_named, struct_newtype, struct_tuple, enum_any)
)]

pub struct ZodTypeDeriveInput {
    pub ident: syn::Ident,
    pub namespace: syn::Path,
    pub attrs: Vec<Attribute>,
    pub generics: Generics,
}

pub struct ZodType {
    pub ident: syn::Ident,
    pub generics: Generics,
    pub config: ContainerConfig,
    pub definition: TokenStream,
    pub dependencies: Vec<Type>,
    pub derive: Derive,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(test, derive(Default))]
pub enum Derive {
    #[cfg_attr(test, default)]
    Request,
    Response,
}

impl ZodType {
    pub fn from_derive_input(orig: &syn::DeriveInput, derive: Derive) -> darling::Result<Self> {
        let input = ZodTypeDeriveInput::from_derive_input(orig)?;

        let cx = serde_derive_internals::Ctxt::new();

        let serde_ast = serde_derive_internals::ast::Container::from_ast(
            &cx,
            &orig,
            serde_derive_internals::Derive::Deserialize,
        )
        .ok_or_else(|| Error::NoSerde)?;

        let serde_attrs = serde_ast.attrs;

        let config = ContainerConfig::new(&serde_attrs, &input.attrs, input.namespace, derive)?;

        if let Err(errors) = cx.check() {
            let mut darling_errors = darling::Error::accumulator();
            for err in errors.into_iter() {
                darling_errors.push(err.into())
            }

            darling_errors.finish()?;
        }

        match serde_ast.data {
            Data::Enum(ref variants) => {
                let dependencies = variants
                    .iter()
                    .map(|v| v.fields.iter().map(|f| f.ty.clone()))
                    .flatten()
                    .collect::<Vec<_>>();

                let definition = Enum {
                    variants,
                    config: &config,
                }
                .expand();

                Ok(Self {
                    ident: input.ident,
                    generics: input.generics,
                    dependencies,
                    definition,
                    config,
                    derive,
                })
            }
            Data::Struct(ref style, ref fields) => {
                let s = Struct {
                    generics: &input.generics,
                    style,
                    fields: FilteredFields::new(
                        fields
                            .into_iter()
                            .map(|f| Field::new(f, derive))
                            .collect::<Result<Vec<_>, _>>()?,
                    ),
                    config: &config,
                    derive,
                };

                let dependencies = s.dependencies();
                let definition = quote!(#s);

                Ok(Self {
                    ident: input.ident,
                    generics: input.generics,
                    dependencies,
                    definition,
                    config,
                    derive,
                })
            }
        }
    }
}

impl<'a> ToTokens for ZodType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let ident = self.ident.clone();

        let definition = &self.definition;
        let dependencies = &self.dependencies;

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let impl_trait = match self.derive {
            Derive::Request => quote!(#zod::core::RequestType),
            Derive::Response => quote!(#zod::core::ResponseType),
        };

        let impl_trait_visitor = match self.derive {
            Derive::Request => quote!(#zod::core::RequestTypeVisitor),
            Derive::Response => quote!(#zod::core::ResponseTypeVisitor),
        };

        let register = match self.derive {
            Derive::Request => {
                quote!(#zod::core::visit_req_dependencies!(ctx, #(#dependencies),*);)
            }
            Derive::Response => {
                quote!(#zod::core::visit_res_dependencies!(ctx, #(#dependencies),*);)
            }
        };

        tokens.extend(quote! {
            impl #impl_generics #impl_trait for #ident #ty_generics #where_clause {
                const AST: #zod::core::ast::Definition = #definition;
            }

            impl #impl_generics #impl_trait_visitor for #ident #ty_generics #where_clause {
                fn register(ctx: &mut #zod::core::DependencyMap)
                where
                    Self: 'static,
                {
                    #register
                }
            }
        })
    }
}
