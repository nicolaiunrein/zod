use darling::FromAttributes;
use darling::{ast::Data, FromDeriveInput};
use proc_macro2::TokenStream;
use quote::quote;
use serde_derive_internals::ast::Container;
use syn::{Attribute, Generics};

use crate::docs::RustDocs;
use crate::r#enum::Enum;
use crate::r#struct::Struct;
use crate::utils::get_zod;
use crate::{field::Field, r#enum::EnumVariant};

#[derive(FromDeriveInput)]
#[darling(
    attributes(zod),
    forward_attrs(allow, doc, cfg),
    supports(struct_named, struct_newtype, struct_tuple, enum_any)
)]

pub struct ZodNode {
    pub ident: syn::Ident,
    pub data: Data<EnumVariant, Field>,
    pub namespace: syn::Path,
    pub attrs: Vec<Attribute>,
    pub generics: Generics,
}

impl ZodNode {
    pub fn expand(self, container: &Container) -> TokenStream {
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

        let docs = RustDocs::from_attributes(&self.attrs).unwrap();

        let definition = match self.data {
            Data::Enum(variants) => Enum {
                variants,
                container,
                docs: &docs,
            }
            .expand(),

            Data::Struct(fields) => {
                let s = Struct {
                    ident: self.ident,
                    ns: self.namespace,
                    generics: self.generics.clone(),
                    fields,
                    docs: &docs,
                };

                quote!(#s)
            }
        };

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        quote! {
            impl #impl_generics #zod::core::ast::Node for #ident #ty_generics #where_clause {
                const DEFINITION: #zod::core::ast::Definition = #definition;
            }

            impl #impl_generics #zod::core::Register for #ident #ty_generics #where_clause {
                fn register(ctx: &mut #zod::core::DependencyMap)
                where
                    Self: 'static,
                {
                    #zod::core::register_dependencies!(ctx, #(#deps),*);
                }
            }

        }
    }
}
