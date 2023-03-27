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
    pub data: Data<darling::util::Ignored, BackendField>,
}

impl ToTokens for BackendInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let backend = BackendImpl { ident: &self.ident };
        let variants = match &self.data {
            Data::Enum(_) => unreachable!(),
            Data::Struct(fields) => fields,
        };

        let req = Request {
            ident: &self.ident,
            variants,
        };

        let output = quote! {
            const _: () = {
                #backend

                #req

            };
        };

        tokens.extend(output);
    }
}

pub struct BackendImpl<'a> {
    ident: &'a Ident,
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

        let ident = self.ident;

        let output = quote! {

            #[#async_trait]
            impl #backend for #ident {
                async fn handle_request(&mut self, req: #request, sender: #sender, subscribers: &mut #subscriber_map) {
                    match req {
                        #request::Exect {id, value} => {
                             match #serde_json::from_value::<#req_ident>(value) {
                                Ok(evt) => {
                                    if let Some(jh) = evt.call(id, self, sender).await {
                                        subscribers.insert(id, jh);
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
                            if let Some(jh) = subscribers.remove(&id) {
                                jh.abort();
                            }
                        }
                    }
                }
            }
        };

        tokens.extend(output)
    }
}

pub struct Request<'a> {
    ident: &'a Ident,
    variants: &'a Fields<BackendField>,
}

#[derive(FromField, Clone)]
pub struct BackendField {
    pub ty: Type,
}

impl<'a> ToTokens for BackendField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let zod = get_zod();
        let rpc_namespace = quote!(#zod::core::rpc::RpcNamespace);
        let ty = &self.ty;
        let variant_ident = variant_ident_from_ty(ty).unwrap();
        let output = quote_spanned!(ty.span() => #variant_ident(<#ty as #rpc_namespace>::Req));

        tokens.extend(output)
    }
}

impl<'a> ToTokens for Request<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let zod = get_zod();
        let variants = self.variants.iter();

        let ident = self.ident;

        let output = quote! {

            #[derive(#zod::__private::serde::Deserialize, Debug)]
            #[serde(tag = "namespace")]
            enum #ident {
                #(#variants),*
            }
        };

        tokens.extend(output)
    }
}
