use crate::config::ContainerConfig;
use crate::config::Derive;
use crate::config::FieldConfig;
use crate::error::Error;
use crate::field::FilteredFields;
use crate::r#enum::EnumExport;
use crate::r#struct::StructExport;
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

struct ZodTypeDeriveInput {
    pub ident: syn::Ident,
    pub namespace: syn::Path,
    pub attrs: Vec<Attribute>,
    pub generics: Generics,
}

pub(crate) struct ZodType {
    pub ident: syn::Ident,
    pub generics: Generics,
    pub definition: TokenStream,
    pub dependencies: Vec<Type>,
    pub derive: Derive,
}

impl ZodType {
    pub(crate) fn from_derive_input(
        orig: &syn::DeriveInput,
        derive: Derive,
    ) -> darling::Result<Self> {
        let input = ZodTypeDeriveInput::from_derive_input(orig)?;

        let cx = serde_derive_internals::Ctxt::new();

        let serde_ast = serde_derive_internals::ast::Container::from_ast(
            &cx,
            orig,
            serde_derive_internals::Derive::Deserialize,
        )
        .ok_or(Error::NoSerde)?;

        let serde_attrs = serde_ast.attrs;

        let config = ContainerConfig::new(&serde_attrs, &input.attrs, input.namespace, derive)?;

        if let Err(errors) = cx.check() {
            let mut darling_errors = darling::Error::accumulator();
            for err in errors.into_iter() {
                darling_errors.push(err.into())
            }

            darling_errors.finish()?;
        }

        let generic_idents = orig
            .generics
            .params
            .iter()
            .filter_map(|param| match param {
                syn::GenericParam::Type(ty) => Some(&ty.ident),
                syn::GenericParam::Lifetime(_) => None,
                syn::GenericParam::Const(_) => None,
            })
            .collect::<Vec<_>>();

        let (dependencies, definition) = match serde_ast.data {
            Data::Enum(ref variants) => {
                let dependencies = variants
                    .iter()
                    .flat_map(|v| v.fields.iter().map(|f| f.ty.clone()))
                    .collect::<Vec<_>>();

                let definition = EnumExport {
                    variants,
                    config: &config,
                }
                .expand();
                (dependencies, definition)
            }
            Data::Struct(ref style, ref fields) => {
                let fields = fields
                    .into_iter()
                    .map(|f| Ok((f.ty, FieldConfig::new(&f.attrs, derive)?)))
                    .collect::<Result<_, crate::error::Error>>()?;

                let fields = FilteredFields::new(fields, &generic_idents)?;
                let dependencies = fields.iter().map(|f| f.ty.clone()).collect();

                let struct_export = StructExport {
                    style,
                    fields,
                    config: &config,
                };

                (dependencies, quote!(#struct_export))
            }
        };

        Ok(Self {
            ident: input.ident,
            generics: input.generics,
            dependencies,
            definition,
            derive,
        })
    }
}

impl ToTokens for ZodType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let ident = self.ident.clone();

        let export = &self.definition;
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

        let args = &self
            .generics
            .params
            .iter()
            .filter_map(|param| match param {
                syn::GenericParam::Type(ty) => Some(&ty.ident),
                syn::GenericParam::Lifetime(_) => None,
                syn::GenericParam::Const(_) => None,
            })
            .map(|ident| match self.derive {
                Derive::Request => quote!(#zod::core::ast::Ref::new_req::<#ident>()),
                Derive::Response => quote!(#zod::core::ast::Ref::new_res::<#ident>()),
            })
            .collect::<Vec<_>>();

        tokens.extend(quote! {
            impl #impl_generics #impl_trait for #ident #ty_generics #where_clause {
                const ARGS: &'static [#zod::core::ast::Ref] = &[
                    #(#args),*
                ];
                const EXPORT: #zod::core::ast::Export = #export;
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
