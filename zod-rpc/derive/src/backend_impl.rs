use darling::ast::Fields;
use proc_macro2::TokenStream;
use syn::{parse_quote, spanned::Spanned, Ident, Path};

use crate::args::{get_private, get_zod, BackendField, BackendInput};
use quote::{format_ident, quote, quote_spanned};

pub fn expand(input: BackendInput, fields: Fields<BackendField>) -> TokenStream {
    let __private = get_private();
    let ident = input.ident;
    let req_ident = format_ident!("{}Req", ident);

    let backend_impl = expand_backend_impl(&ident, &req_ident, &fields);
    let req_enum = expand_req_enum(&req_ident, &fields);
    let req_enum_impl = expand_req_enum_impl(&ident, &req_ident, &fields);

    quote! {
        #backend_impl

        #req_enum

        #req_enum_impl


    }
}

fn expand_backend_impl(
    ident: &Ident,
    req_ident: &Ident,
    fields: &Fields<BackendField>,
) -> TokenStream {
    let zod = get_zod();
    let __private = get_private();

    let namespaces = fields.iter().map(|f| {
        let ty = &f.ty;
        quote_spanned!(ty.span() => <#ty as #zod::Namespace>::NAME)
    });

    quote! {
        #[async_trait::async_trait]
        impl #__private::server::Backend for #ident {
            fn is_member_of_self(member: &'static #__private::codegen::RpcMember) -> bool {
                static NAMES: &'static [&'static str] = &[#(#namespaces),*];
                NAMES.contains(&member.ns_name())
            }

            async fn handle_request(
                &mut self,
                req: #__private::Request,
                sender: #__private::ResponseSender,
                subscribers: &mut #__private::server::SubscriberMap,
            ) {
                match req {
                    #__private::Request::Exec { id, value } => {
                        match #__private::serde_json::from_value::<#req_ident>(value) {
                            ::std::result::Result::<_, _>::Ok(evt) => {
                                if let Some(jh) = evt.call(id, self, sender).await {
                                    subscribers.insert(id, jh);
                                }
                            }
                            ::std::result::Result::<_, _>::Err(err) => {
                                let _ = sender
                                    .unbounded_send(#__private::Response::error(id, err))
                                    .ok();
                            }
                        }
                    }
                    #__private::Request::CancelStream { id } => {
                        if let Some(jh) = subscribers.remove(&id) {
                            jh.abort();
                        }
                    }
                }
            }
        }
    }
}

pub fn expand_req_enum(ident: &Ident, fields: &Fields<BackendField>) -> TokenStream {
    let __private = get_private();
    let req_variants = fields.iter().map(|f| {
        let ty = &f.ty;
        let variant_ident = variant_ident_from_ty(ty);
        quote_spanned!(ty.span() => #variant_ident(<#ty as #__private::codegen::Rpc>::Req))
    });
    quote! {

        #[derive(#__private::serde::Deserialize, Debug)]
        #[serde(tag = "namespace")]
        enum #ident {
            #(#req_variants),*
        }
    }
}

pub fn expand_req_enum_impl(
    backend_ident: &Ident,
    req_ident: &Ident,
    fields: &Fields<BackendField>,
) -> TokenStream {
    let __private = get_private();

    let req_call_dispatch = fields.iter().enumerate().map(|(index, f)| {
        let ty = &f.ty;
        let variant_ident = variant_ident_from_ty(ty);
        let field_or_index = f.ident.clone().map(|ident| quote!(#ident)).unwrap_or_else(|| {
            let index = syn::Index::from(index);
            quote!(#index)
        });

        quote_spanned!(ty.span() => #req_ident :: #variant_ident(req) => req.call(id, &mut backend.#field_or_index, sender).await)
    });

    quote! {
        impl #req_ident {
            pub async fn call(
                self,
                id: usize,
                backend: &mut #backend_ident,
                sender: #__private::ResponseSender,
            ) -> ::std::option::Option<#__private::tokio::task::JoinHandle<()>> {
                match self {
                    #(#req_call_dispatch),*
                }
            }
        }
    }
}

fn variant_ident_from_ty(ty: &syn::Type) -> Ident {
    let p: Path = parse_quote!(#ty);
    p.segments.last().expect("one segment").ident.clone()
}
