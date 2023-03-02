use proc_macro2::TokenStream;

use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use syn::Ident;

use crate::args::{self, get_private, get_zod, RpcArg, RpcItemKind};

pub fn expand(input: args::RpcInput) -> TokenStream {
    let __private = get_private();

    let req_ident = input.req_ident();
    let ident = input.ident;

    let req_variant_defs = input.items.iter().map(expand_req_variant_decl);
    let req_variant_impls = input.items.iter().map(expand_req_variant_impl);
    let inventory_submits = input
        .items
        .iter()
        .map(|item| expand_inventory_submit(&ident, item));

    quote_spanned! {
        ident.span() =>
        const _: () = {
            impl #__private::codegen::RpcNamespace for #ident {
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


            #(#inventory_submits)*
        };
    }
}

pub fn expand_inventory_submit(ns_ident: &Ident, item: &args::RpcItem) -> TokenStream {
    let __private = get_private();
    let zod = get_zod();
    let name = item.ident.to_string();

    let args = item
        .arg_types
        .iter()
        .map(|RpcArg { ty, name }| quote!(#__private::codegen::RpcArgument::new::<#ty>(#name)));

    match (&item.kind, &item.output) {
        (RpcItemKind::Method, args::OutputType::ImplItem(_)) => {
            abort!(
                item.ident.span(),
                "zod: namespace methods returning a stream are not allowed to be async"
            )
        }

        (RpcItemKind::Method, args::OutputType::Concrete(t)) => {
            quote_spanned! { item.ident.span() =>
                #__private::inventory::submit!(#__private::codegen::RpcMember::Method {
                    ns_name: <#ns_ident as #zod::Namespace>::NAME,
                    name: #name,
                    args: &|| vec![
                        #(#args),*
                    ],
                    res: &|| <#t as ::zod::ZodType>::type_def().to_string(),
                });

            }
        }

        (RpcItemKind::Stream, args::OutputType::ImplItem(t)) => {
            quote_spanned! { item.ident.span() =>
                #__private::inventory::submit!(::zod::rpc::__private::codegen::RpcMember::Stream {
                    ns_name: <#ns_ident as ::zod::Namespace>::NAME,
                    name: #name,
                    args: &|| vec![
                        #(#args),*
                    ],
                    res: &|| <#t as ::zod::ZodType>::type_def().to_string(),
                });
            }
        }
        (RpcItemKind::Stream, args::OutputType::Concrete(t)) => {
            quote_spanned! { item.ident.span() =>
                #__private::inventory::submit!(::zod::rpc::__private::codegen::RpcMember::Stream {
                    ns_name: <#ns_ident as ::zod::Namespace>::NAME,
                    name: #name,
                    args: &|| vec![
                        #(#args),*
                    ],
                    res: &|| <<#t as ::zod::rpc::__private::futures::Stream>::Item as ::zod::ZodType>::type_def().to_string(),
                });
            }
        }
    }
}

pub fn expand_req_variant_decl(item: &args::RpcItem) -> TokenStream {
    let ident = &item.ident;
    let arg_types = item.arg_types.iter().map(|RpcArg { ty, .. }| quote!(#ty,));

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
        RpcItemKind::Method => expand_req_variant_impl_method(ident, expanded_args),
        RpcItemKind::Stream => expand_req_variant_impl_stream(ident, expanded_args),
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

    quote_spanned! {
        ident.span() =>
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

    quote_spanned! { ident.span() =>
            let s = ctx.#ident(#(#expanded_args),*);
            Some(#__private::tokio::spawn(async move {
                #__private::futures::pin_mut!(s);
                while let ::std::option::Option::Some(evt) =
                    {
                        let item = #__private::futures::StreamExt::next(&mut s);
                        item.await
                    }
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