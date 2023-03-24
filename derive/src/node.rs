use crate::config::Config;
use crate::r#enum::Enum;
use crate::r#struct::Struct;
use crate::utils::get_zod;
use crate::{field::Field, r#enum::EnumVariant};
use darling::ToTokens;
use darling::{ast::Data, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Generics};

#[derive(FromDeriveInput)]
#[darling(
    attributes(zod),
    forward_attrs(allow, doc, cfg),
    supports(struct_named, struct_newtype, struct_tuple, enum_any)
)]

pub struct ZodNodeInput {
    pub ident: syn::Ident,
    pub data: Data<EnumVariant, Field>,
    pub namespace: syn::Path,
    pub attrs: Vec<Attribute>,
    pub generics: Generics,
}

pub struct ZodNode {
    pub ident: syn::Ident,
    pub data: Data<EnumVariant, Field>,
    pub generics: Generics,
    pub config: Config,
}

impl FromDeriveInput for ZodNode {
    fn from_derive_input(orig: &syn::DeriveInput) -> darling::Result<Self> {
        let cx = serde_derive_internals::Ctxt::new();

        let input = ZodNodeInput::from_derive_input(orig)?;

        let config = Config::new(&orig, input.namespace)?;

        Ok(Self {
            ident: input.ident,
            data: input.data,
            generics: input.generics,
            config,
        })
    }
}

impl ToTokens for ZodNode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let ident = self.ident.clone();

        let deps = match self.data {
            Data::Enum(ref variants) => variants
                .iter()
                .map(|v| v.fields.iter().map(|f| f.ty.clone()))
                .flatten()
                .collect::<Vec<_>>(),

            Data::Struct(ref fields) => fields.iter().map(|f| f.ty.clone()).collect::<Vec<_>>(),
        };

        let definition = match &self.data {
            Data::Enum(variants) => Enum {
                variants,
                config: &self.config,
            }
            .expand(),

            Data::Struct(fields) => {
                let s = Struct {
                    ident: &self.ident,
                    generics: &self.generics,
                    fields,
                    config: &self.config,
                };

                quote!(#s)
            }
        };

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        tokens.extend(quote! {
            impl #impl_generics #zod::core::Node for #ident #ty_generics #where_clause {
                const AST: #zod::core::ast::Definition = #definition;
            }

            impl #impl_generics #zod::core::Register for #ident #ty_generics #where_clause {
                fn register(ctx: &mut #zod::core::DependencyMap)
                where
                    Self: 'static,
                {
                    #zod::core::register_dependencies!(ctx, #(#deps),*);
                }
            }
        })
    }
}
