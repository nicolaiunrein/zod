use proc_macro2::TokenStream;

use quote::quote;
use syn::Ident;

use crate::args::{self, get_private, RpcItemKind};

pub fn expand(input: args::RpcInput) -> TokenStream {
    let __private = get_private();

    let req_ident = input.req_ident();
    let ident = input.ident;

    let req_variant_defs = input.items.iter().map(expand_req_variant_decl);
    let req_variant_impls = input.items.iter().map(|item| expand_req_variant_impl(item));

    quote! {
        impl #__private::codegen::Rpc for #ident {
            type Req = #req_ident;
        }

        #[derive(#__private::serde::Deserialize, Debug)]
        #[serde(tag = "method")]
        #[allow(non_camel_case_types)]
        #[allow(non_snake_case)]
        #[allow(non_upper_case_globals)]
        pub enum #req_ident {
            #(#req_variant_defs),*
        }

        impl #req_ident {
            pub async fn call(
                self,
                id: usize,
                ctx: &mut #ident,
                sender: #__private::ResponseSender,
            ) -> ::std::option::Option<#__private::tokio::task::JoinHandle<()>> {
                match self {
                    #(#req_variant_impls),*
                }
            }
        }
    }
}

pub fn expand_req_variant_decl(item: &args::RpcItem) -> TokenStream {
    let ident = &item.ident;
    let arg_types = item.arg_types.iter().map(|arg| quote!(#arg,));
    quote! {
        #ident { args: (#(#arg_types)*) }
    }
}
pub fn expand_req_variant_impl(input: &args::RpcItem) -> TokenStream {
    let __private = get_private();
    let ident = &input.ident;

    let expanded_args = input
        .arg_types
        .iter()
        .enumerate()
        .map(|(i, _)| syn::Index::from(i))
        .map(|i| quote!(args.#i));

    let inner = match input.kind {
        RpcItemKind::Method => expand_req_variant_impl_method(&ident, expanded_args),
        RpcItemKind::Stream => expand_req_variant_impl_stream(&ident, expanded_args),
    };

    quote! {
        Self::#ident { args } => {
            #inner
        }
    }
}

pub fn expand_req_variant_impl_method(
    ident: &Ident,
    expanded_args: impl Iterator<Item = TokenStream>,
) -> TokenStream {
    let __private = get_private();

    quote! {
        let res = ctx.#ident(#(#expanded_args),*).await;

        sender
            .unbounded_send(#__private::Response::method(id, res))
            .unwrap();
        None
    }
}

pub fn expand_req_variant_impl_stream(
    ident: &Ident,
    expanded_args: impl Iterator<Item = TokenStream>,
) -> TokenStream {
    let __private = get_private();
    quote! {
            let s = ctx.#ident(#(#expanded_args),*);
            Some(#__private::tokio::spawn(async move {
                #__private::futures::pin_mut!(s);
                while let ::std::option::Option::Some(evt) =
                    #__private::futures::StreamExt::next(&mut s).await
                {
                    if let ::std::result::Result::<_, _>::Err(err) = sender
                        .unbounded_send(#__private::Response::stream(id, evt))
                    {
                        #__private::tracing::warn!(?err, "Failed to emit event");
                        break;
                    }
                }
            }))
    }
}
