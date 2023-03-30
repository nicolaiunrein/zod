use crate::utils::get_zod;
use darling::ast::Data;
use darling::{FromDeriveInput, FromField, ToTokens};
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

impl ToTokens for BackendInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let zod = get_zod();

        let ident = &self.ident;

        let variants = match &self.data {
            Data::Enum(_) => unreachable!(),
            Data::Struct(fields) => fields,
        };

        let types = variants.iter().map(|f| &f.ty).collect::<Vec<_>>();

        let variants = types
            .iter()
            .enumerate()
            .map(|(i, ty)| {
                let ident = format_ident!("Ns{}", i);
                (ident, ty)
            })
            .collect::<Vec<_>>();

        let variants_inner = variants
            .iter()
            .map(|(ident, ty)| quote!(#ident(<#ty as #zod::core::rpc::RpcNamespace>::Req)));

        let inner_match =
            variants
                .iter()
                .map(|(ident, _)| ident)
                .enumerate()
                .map(|(index, ident)| {
                    let index = syn::Index::from(index);
                    quote!(Inner::#ident(req) => {
                        if let Some(jh) = self.#index.process(req, sender, *id).await {
                            subscribers.insert(*id, jh);
                        }
                    })
                });

        let output = quote! {
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
                                #[derive(Deserialize)]
                                enum Inner {
                                    #(#variants_inner),*
                                }

                                match #zod::__private::serde_json::from_value::<Inner>(value) {
                                    Ok(inner) => match inner {
                                        #(#inner_match),*
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

        tokens.extend(output);
    }
}
