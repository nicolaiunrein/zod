use crate::error::Error;
use darling::ToTokens;
use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::format_ident;
use syn::{parse_quote, Ident, ImplItem, ImplItemMethod, ItemImpl, Type};

pub struct RpcInput {
    pub ident: syn::Ident,
    pub items: Vec<RpcItem>,
}

pub struct RpcItem {
    pub ident: syn::Ident,
    pub arg_types: Vec<RpcArg>,
    pub kind: RpcItemKind,
    pub output: Box<Type>,
}

pub struct RpcArg {
    pub name: String,
    pub ty: Box<Type>,
}

pub enum RpcItemKind {
    Method,
    Stream,
}

impl RpcInput {
    pub(crate) fn req_ident(&self) -> Ident {
        format_ident!("{}Req", self.ident)
    }
}

impl TryFrom<ItemImpl> for RpcInput {
    type Error = syn::Error;

    fn try_from(input: ItemImpl) -> Result<Self, Self::Error> {
        let self_ty = input.self_ty;
        let ident: Ident = parse_quote!(#self_ty);
        Ok(Self {
            ident,
            items: input
                .items
                .into_iter()
                .filter_map(|item| match item {
                    ImplItem::Method(method) => Some(method),
                    _ => None,
                })
                .map(|item| RpcItem::try_from(item))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl ToTokens for RpcInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let req_ident = self.req_ident();
        let ident = &self.ident;

        // let req_variant_defs = self.items.iter().map(expand_req_variant_decl);
        // let req_variant_impls = self.items.iter().map(expand_req_variant_impl);

        // let output = quote_spanned! {
        // ident.span() =>
        // const _: () = {
        // impl #__private::codegen::RpcNamespace for #ident {
        // type Req = #req_ident;
        // }

        // #[derive(#__private::serde::Deserialize, Debug)]
        // #[serde(tag = "method")]
        // #[allow(non_camel_case_types)]
        // #[allow(non_snake_case)]
        // #[allow(non_upper_case_globals)]
        // pub enum #req_ident {
        // #(#req_variant_defs),*
        // }

        // impl #req_ident {
        // #[allow(dead_code)]
        // #[allow(unused_variables)]
        // pub async fn call(
        // self,
        // id: usize,
        // ctx: &mut #ident,
        // sender: #__private::ResponseSender,
        // ) -> ::std::option::Option<#__private::tokio::task::JoinHandle<()>> {
        // match self {
        // #(#req_variant_impls),*
        // }
        // }
        // }
        // };
        // };

        tokens.extend(quote::quote!());
    }
}

impl TryFrom<ImplItemMethod> for RpcItem {
    type Error = syn::Error;

    fn try_from(input: ImplItemMethod) -> Result<Self, Self::Error> {
        let sig = input.sig;
        let ident = sig.ident;
        let is_async = sig.asyncness.is_some();

        let kind = if is_async {
            RpcItemKind::Method
        } else {
            RpcItemKind::Stream
        };

        let output = match (&kind, sig.output) {
            (RpcItemKind::Method, syn::ReturnType::Default) => {
                parse_quote!(())
            }
            (RpcItemKind::Stream, syn::ReturnType::Default) => {
                return Err(Error::NonAsyncReturningDefault(ident.span()).into());
            }
            (RpcItemKind::Method, syn::ReturnType::Type(_, t))
            | (RpcItemKind::Stream, syn::ReturnType::Type(_, t)) => t,
        };

        if let Some(receiver) = sig.inputs.iter().find_map(|arg| match arg {
            syn::FnArg::Receiver(inner) => Some(inner),
            _ => None,
        }) {
            match (receiver.mutability, &receiver.reference) {
                (Some(_), Some((_, None))) => {}
                (Some(_), Some((_, Some(lifetime)))) => {
                    return Err(Error::NamespaceLifetimes(lifetime.span()).into());
                }
                (None, None) => return Err(Error::owned_self(receiver.self_token.span).into()),
                (None, Some((and, _))) => return Err(Error::shared_self(and.span).into()),
                (Some(_), None) => return Err(Error::mut_self(receiver.self_token.span).into()),
            }
        } else {
            return Err(Error::NoSelf(ident.span()).into());
        }

        let arg_types = sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(t) => Some(RpcArg {
                    ty: t.ty.clone(),
                    name: match t.pat.as_ref() {
                        syn::Pat::Ident(ident) => ident.ident.to_string(),
                        _ => abort_call_site!("Expected an ident, got {:?}", t.pat),
                    },
                }),
            })
            .collect();

        Ok(Self {
            ident,
            arg_types,
            kind,
            output,
        })
    }
}
