use crate::utils::get_zod;
use darling::ast::Data;
use darling::{FromDeriveInput, FromField, ToTokens};
use proc_macro2::TokenStream;
use quote::quote;
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
}

impl ToTokens for BackendInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let ident = &self.ident;
        let types = self.variant_types();
        let indices = types
            .iter()
            .enumerate()
            .map(|(index, _)| syn::Index::from(index));

        let expanded = quote! {
            const _: () = {
                #[#zod::__private::async_trait::async_trait]
                impl #zod::core::rpc::server::Backend for #ident {
                    const AST: &'static [&'static [#zod::core::ast::rpc::RpcRequest]] = &[
                        #(<#types as #zod::core::rpc::RpcNamespace>::AST),*
                    ];


                    async fn forward_request(
                        &mut self,
                        connection_id: usize,
                        req: #zod::core::rpc::Request,
                        sender: #zod::core::rpc::ResponseSender,
                        subscribers: &mut #zod::core::rpc::server::SubscriberMap,
                    ) {
                        match req {
                            #zod::core::rpc::Request::Exec { id, value } => {

                                match #zod::__private::serde_json::from_value::<#zod::core::rpc::AnyNsRequest>(value) {
                                    Ok(req) => match req.ns.as_str() {
                                        #(<#types as #zod::core::Namespace>::NAME => match #zod::__private::serde_json::from_value::<<#types as #zod::core::rpc::RpcNamespace>::Req>(req.inner) {
                                            Ok(inner) => {
                                                if let Some(jh) = #zod::core::rpc::RpcNamespace::process(&mut self.#indices, inner, sender, *id).await {
                                                    subscribers.insert((connection_id, *id), jh);
                                                }
                                            },
                                            Err(err) => {
                                                let _ = sender
                                                    .unbounded_send(#zod::core::rpc::Response::error(id, err))
                                                    .ok();
                                            }
                                        }),*
                                        ns => {
                                                let _ = sender
                                                    .unbounded_send(#zod::core::rpc::Response::error(id, #zod::core::rpc::Error::UnknownNamespace(::std::string::String::from(ns))))
                                                    .ok();
                                        }
                                    },
                                    Err(err) => {
                                        let _ = sender
                                            .unbounded_send(#zod::core::rpc::Response::error(id, err))
                                            .ok();
                                    }

                                };
                            },

                            #zod::core::rpc::Request::CancelStream { id } => {
                                subscribers.remove(&(connection_id, *id));
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
