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

pub struct ZodNodeInput {
    pub ident: syn::Ident,
    pub namespace: syn::Path,
    pub attrs: Vec<Attribute>,
    pub generics: Generics,
}

pub struct ZodNode {
    pub ident: syn::Ident,
    pub generics: Generics,
    pub config: ContainerConfig,
    pub definition: TokenStream,
    pub dependencies: Vec<Type>,
}

impl ZodNode {
    pub fn from_derive_input(orig: &syn::DeriveInput) -> darling::Result<Self> {
        let input = ZodNodeInput::from_derive_input(orig)?;

        let cx = serde_derive_internals::Ctxt::new();

        let serde_ast = serde_derive_internals::ast::Container::from_ast(
            &cx,
            &orig,
            serde_derive_internals::Derive::Deserialize,
        )
        .ok_or_else(|| Error::NoSerde)?;

        let serde_attrs = serde_ast.attrs;

        let config = ContainerConfig::new(&serde_attrs, &input.attrs, input.namespace)?;

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
                })
            }
            Data::Struct(ref style, ref fields) => {
                let s = Struct {
                    generics: &input.generics,
                    style,
                    fields: FilteredFields::new(
                        fields
                            .into_iter()
                            .map(Field::new)
                            .collect::<Result<Vec<_>, _>>()?,
                    ),
                    config: &config,
                };

                let dependencies = s.dependencies();
                let definition = quote!(#s);

                Ok(Self {
                    ident: input.ident,
                    generics: input.generics,
                    dependencies,
                    definition,
                    config,
                })
            }
        }
    }
}

impl<'a> ToTokens for ZodNode {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let ident = self.ident.clone();

        let definition = &self.definition;
        let dependencies = &self.dependencies;

        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        tokens.extend(quote! {
            impl #impl_generics #zod::core::InputType for #ident #ty_generics #where_clause {
                const AST: #zod::core::ast::Definition = #definition;
            }

            impl #impl_generics #zod::core::Register for #ident #ty_generics #where_clause {
                fn register(ctx: &mut #zod::core::DependencyMap)
                where
                    Self: 'static,
                {
                    #zod::core::register_dependencies!(ctx, #(#dependencies),*);
                }
            }
        })
    }
}
