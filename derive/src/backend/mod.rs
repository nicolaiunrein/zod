use darling::ast::{Data, Fields};
use darling::{FromDeriveInput, FromField, ToTokens};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{parse_quote, Ident, Path, Type};

use crate::utils::get_zod;

fn req_ident(ident: &Ident) -> Ident {
    format_ident!("{}Request", ident)
}

fn variant_ident_from_ty(ty: &syn::Type) -> Option<Ident> {
    let p: Path = parse_quote!(#ty);
    p.get_ident().cloned()
}

#[derive(FromDeriveInput)]
pub struct BackendInput {
    pub ident: syn::Ident,
    pub data: Data<darling::util::Ignored, BackendVariant>,
}

impl ToTokens for BackendInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let variants = match &self.data {
            Data::Enum(_) => unreachable!(),
            Data::Struct(fields) => fields,
        };

        let backend = BackendImpl {
            ident: &self.ident,
            variants,
        };

        let req = BackendRequest {
            ident: &self.ident,
            variants,
        };

        let output = quote! {
            const _: () = {
                #req

                #backend
            };
        };

        tokens.extend(output);
    }
}

pub struct BackendImpl<'a> {
    ident: &'a Ident,
    variants: &'a Fields<BackendVariant>,
}

impl<'a> ToTokens for BackendImpl<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let zod = get_zod();
        let async_trait = quote!(#zod::__private::async_trait::async_trait);
        let backend = quote!(#zod::core::rpc::server::Backend);
        let request = quote!(#zod::core::rpc::Request);
        let sender = quote!(#zod::core::rpc::ResponseSender);
        let response = quote!(#zod::core::rpc::Response);
        let serde_json = quote!(#zod::__private::serde_json);
        let req_ident = req_ident(self.ident);
        let subscriber_map = quote!(#zod::core::rpc::server::SubscriberMap);
        let res_type_visitor = quote!(#zod::core::ResponseTypeVisitor);
        let req_type_visitor = quote!(#zod::core::RequestTypeVisitor);
        let dep_map = quote!(#zod::core::DependencyMap);
        let variants: Vec<_> = self.variants.iter().collect();

        let ident = self.ident;

        let output = quote! {

            #[#async_trait]
            impl #backend for #ident {
                async fn handle_request(&mut self, req: #request, sender: #sender, subscribers: &mut #subscriber_map) {
                    match req {
                        #request::Exec {id, value} => {
                             match #serde_json::from_value::<#req_ident>(value) {
                                Ok(evt) => {
                                    if let Some(jh) = evt.call(id, self, sender).await {
                                        subscribers.insert(id.into(), jh);
                                    }
                                }

                                Err(err) => {
                                    let _ = sender
                                        .unbounded_send(#response::error(id, err))
                                        .ok();
                                }
                            }
                        },

                         #request::CancelStream { id } => {
                            subscribers.remove(&id);
                        }
                    }
                }
            }

            impl #res_type_visitor for #ident {
                fn register(ctx: &mut #dep_map)
                where
                    Self: 'static,
                {
                    #(<#variants as #res_type_visitor>::register(ctx));*
                }
            }
            impl #req_type_visitor for #ident {
                fn register(ctx: &mut #dep_map)
                where
                    Self: 'static,
                {
                    #(<#variants as #req_type_visitor>::register(ctx));*
                }
            }

        };

        tokens.extend(output)
    }
}

pub struct BackendRequest<'a> {
    ident: &'a Ident,
    variants: &'a Fields<BackendVariant>,
}

impl<'a> ToTokens for BackendRequest<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let zod = get_zod();
        let variants = self.variants.iter();

        let ident = req_ident(self.ident);

        let output = quote! {

            #[derive(#zod::__private::serde::Deserialize)]
            #[serde(tag = "namespace")]
            enum #ident {
                #(#variants),*
            }
        };

        tokens.extend(output)
    }
}

#[derive(FromField, Clone)]
pub struct BackendVariant {
    pub ty: Type,
}

impl<'a> ToTokens for BackendVariant {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let zod = get_zod();
        let rpc_namespace = quote!(#zod::core::rpc::RpcNamespace);
        let ty = &self.ty;
        let variant_ident = variant_ident_from_ty(ty).unwrap();
        let output = quote_spanned!(ty.span() => #variant_ident(<#ty as #rpc_namespace>::Req));

        tokens.extend(output)
    }
}
