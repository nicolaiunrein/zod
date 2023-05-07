use crate::utils::get_zod;
use darling::ast::Data;
use darling::{FromDeriveInput, FromField, ToTokens};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Type;

#[derive(FromDeriveInput)]
pub(crate) struct BackendInput {
    pub(crate) ident: syn::Ident,
    pub(crate) data: Data<darling::util::Ignored, BackendVariant>,
}

#[derive(FromField, Clone)]
pub(crate) struct BackendVariant {
    pub(crate) ty: Type,
}

impl BackendVariant {
    fn expand_match_arm(&self, index: usize) -> TokenStream {
        let ident = Self::format_ident(index);
        let index = syn::Index::from(index);
        let zod = get_zod();

        quote!(Inner::#ident{ req, .. } => {
            if let Some(jh) = #zod::core::rpc::RpcNamespace::process(&mut self.#index, req, sender, *id).await {
                subscribers.insert(*id, jh);
            }
        })
    }

    fn expand_enum_variant(&self, index: usize) -> TokenStream {
        let ty = &self.ty;
        let zod = get_zod();

        let ident = Self::format_ident(index);
        quote!(#ident{
            #[serde(flatten)]
            req: <#ty as #zod::core::rpc::RpcNamespace>::Req,
            ns: #zod::core::rpc::RpcNamespaceName<#ty>,
        })
    }

    fn format_ident(index: usize) -> syn::Ident {
        format_ident!("Ns{}", index)
    }
}

impl BackendInput {
    fn variants(&self) -> &darling::ast::Fields<BackendVariant> {
        match &self.data {
            Data::Enum(_) => unreachable!(),
            Data::Struct(fields) => fields,
        }
    }

    fn variant_types(&self) -> Vec<&syn::Type> {
        self.variants().iter().map(|f| &f.ty).collect()
    }

    fn match_arms(&self) -> Vec<TokenStream> {
        self.variants()
            .iter()
            .enumerate()
            .map(|(index, variant)| variant.expand_match_arm(index))
            .collect()
    }

    fn enum_variants(&self) -> Vec<TokenStream> {
        self.variants()
            .iter()
            .enumerate()
            .map(|(index, variant)| variant.expand_enum_variant(index))
            .collect()
    }
}

impl ToTokens for BackendInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let ident = &self.ident;
        let types = self.variant_types();
        let match_arms = self.match_arms();
        let enum_variants = self.enum_variants();

        let expanded = quote! {
            const _: () = {
                #[#zod::__private::async_trait::async_trait]
                impl #zod::core::rpc::server::Backend for #ident {
                    const AST: &'static [&'static [#zod::core::ast::rpc::RpcRequest]] = &[
                        #(<#types as #zod::core::rpc::RpcNamespace>::AST),*
                    ];


                    async fn forward_request(
                        &mut self,
                        req: #zod::core::rpc::Request,
                        sender: #zod::core::rpc::ResponseSender,
                        subscribers: &mut #zod::core::rpc::server::SubscriberMap,
                    ) {
                        match req {
                            #zod::core::rpc::Request::Exec { id, value } => {
                                #[derive(#zod::__private::serde::Deserialize)]
                                #[serde(untagged)]
                                enum Inner {
                                    #(#enum_variants),*
                                }

                                match #zod::__private::serde_json::from_value::<Inner>(value) {
                                    Ok(inner) => match inner {
                                        #(#match_arms),*
                                    },

                                    Err(err) => {
                                        let _ = sender
                                            .unbounded_send(zod_core::rpc::Response::error(id, err))
                                            .ok();
                                    }
                                }
                            },

                            #zod::core::rpc::Request::CancelStream { id } => {
                                subscribers.remove(&id);
                            }
                        }
                    }
                }

                impl #zod::core::RequestTypeVisitor for #ident {
                    fn register(ctx: &mut #zod::core::DependencyMap)
                    where
                        Self: 'static,
                    {
                        #(<<#types as #zod::core::rpc::RpcNamespace>::Req as #zod::core::RequestTypeVisitor>::register(ctx));*
                    }
                }

                impl #zod::core::ResponseTypeVisitor for #ident {
                    fn register(ctx: &mut #zod::core::DependencyMap)
                    where
                        Self: 'static,
                    {
                        #(<<#types as #zod::core::rpc::RpcNamespace>::Req as #zod::core::ResponseTypeVisitor>::register(ctx));*
                    }
                }
            };
        };

        tokens.extend(expanded);
    }
}
