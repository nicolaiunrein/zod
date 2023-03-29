use darling::ToTokens;
use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::{parse_quote, Ident, ImplItem, ImplItemMethod, ItemImpl, Type};

use crate::error::Error;
use crate::utils::get_zod;

pub struct RpcInput {
    ident: syn::Ident,
    items: Vec<RpcItem>,
}

pub struct RpcItem {
    ident: syn::Ident,
    method_args: Vec<RpcArg>,
    kind: RpcItemKind,
}

pub struct RpcArg {
    name: String,
    ty: Box<Type>,
}

pub enum RpcItemKind {
    Method(Box<Type>),
    Stream(Box<Type>),
}

impl ToTokens for RpcInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let ident = &self.ident;

        let enum_variants = self
            .items
            .iter()
            .map(|item| {
                let field_ident = &item.ident;
                let types = item.method_args.iter().map(|arg| &arg.ty);

                // todo
                let maybe_comma = if item.method_args.len() == 1 {
                    quote!(,)
                } else {
                    quote!()
                };

                quote!(#field_ident {args: (#(#types),* #maybe_comma) })
            })
            .collect::<Vec<_>>();

        let input_types = self
            .items
            .iter()
            .map(|item| item.method_args.iter().map(|arg| &arg.ty))
            .flatten()
            .collect::<Vec<_>>();

        let rpc_requests = self.items.iter().map(|item| {
            let name = item.ident.to_string();

            let args = item.method_args.iter().map(|arg| {
                let ty = &arg.ty;
                let name = &arg.name;
                quote!(#zod::core::ast::NamedField::new::<#ty>(#name))
            });

            match &item.kind {
                RpcItemKind::Method(output) => {
                    quote! {
                        #zod::core::ast::rpc::RpcRequest {
                            path: #zod::core::ast::Path::new::<#ident>(#name),
                            args: &[#(#args),*],
                            res: <#output as #zod::core::ResponseType>::AST.inline(),
                            kind: #zod::core::ast::rpc::RpcRequestKind::Method,
                        }
                    }
                }
                RpcItemKind::Stream(_) => {
                    let arg_types = item
                        .method_args
                        .iter()
                        .map(|arg| &arg.ty)
                        .collect::<Vec<_>>();

                    let item_ast = ImplStreamItemAstExtractor {
                        ns: &self.ident,
                        method: &item.ident,
                        args: &arg_types,
                    };

                    quote! {
                        #zod::core::ast::rpc::RpcRequest {
                            path: #zod::core::ast::Path::new::<#ident>(#name),
                            args: &[#(#args),*],
                            res: #item_ast.get().inline(),
                            kind: #zod::core::ast::rpc::RpcRequestKind::Stream,
                        }
                    }
                }
            }
        });

        let match_entries = self.items.iter().map(|item| {
            let ident = &item.ident;
            let args = item.method_args.iter().enumerate().map(|(i, _)| {
                let index = syn::Index::from(i);
                quote!(args.#index)
            });

            match &item.kind {
                RpcItemKind::Method(_) => {
                    quote! {
                        NsReq:: #ident { args } => {
                            let res = self.#ident(#(#args),*).await;
                            let _ = sender.unbounded_send(#zod::core::rpc::Response::method(id, res));
                            None
                        }
                    }
                }
                RpcItemKind::Stream(_) => {
                    quote! {
                        NsReq:: #ident { args } => {
                            let mut stream = self.#ident(#(#args),*);
                            let jh = tokio::spawn(async move {
                                #zod::__private::futures::pin_mut!(stream);
                                while let Some(evt) = stream.next().await {
                                    let _ = sender.unbounded_send(#zod::core::rpc::Response::stream(id, evt));
                                }
                            });

                            Some(jh.into())
                        }
                    }
                }
            }
        });

        let output = quote! {
            const _: () = {
                #[derive(#zod::__private::serde::Deserialize)]
                #[serde(tag = "method")]
                #[allow(non_camel_case_types)]
                #[allow(non_snake_case)]
                #[allow(non_upper_case_globals)]
                #[doc(hidden)]
                pub enum NsReq {
                    #(#enum_variants),*
                }

                impl #zod::core::RequestTypeVisitor for NsReq {
                    fn register(ctx: &mut #zod::core::DependencyMap)
                    where
                        Self: 'static,
                    {
                        #(<#input_types as #zod::core::RequestTypeVisitor>::register(ctx));*
                    }
                }

                impl #zod::core::ResponseTypeVisitor for NsReq {
                    fn register(ctx: &mut #zod::core::DependencyMap)
                    where
                        Self: 'static,
                    {

                        // #(<#response_types as #zod::core::ResponseTypeVisitor>::register(ctx));*
                    }
                }

                #[#zod::__private::async_trait::async_trait]
                impl #zod::core::rpc::RpcNamespace for #ident {
                    const AST: &'static [#zod::core::ast::rpc::RpcRequest] = &[
                        #(#rpc_requests),*
                    ];

                    type Req = NsReq;

                    async fn process(
                        &mut self,
                        req: Self::Req,
                        sender: #zod::core::rpc::ResponseSender,
                        id: usize,
                    ) -> Option<#zod::core::rpc::server::StreamHandle> {
                        match req {
                            #(#match_entries),*
                        }
                    }
                }
            };
        };

        tokens.extend(output);
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

impl TryFrom<ImplItemMethod> for RpcItem {
    type Error = syn::Error;

    fn try_from(input: ImplItemMethod) -> Result<Self, Self::Error> {
        let sig = input.sig;
        let ident = sig.ident;
        let is_async = sig.asyncness.is_some();

        let kind = match (is_async, sig.output) {
            (true, syn::ReturnType::Default) => RpcItemKind::Method(parse_quote!(())),
            (false, syn::ReturnType::Default) => {
                return Err(Error::NonAsyncReturningDefault(ident.span()).into());
            }
            (true, syn::ReturnType::Type(_, t)) => RpcItemKind::Method(t),
            (false, syn::ReturnType::Type(_, t)) => RpcItemKind::Stream(t),
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

        let method_args = sig
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
            method_args,
            kind,
        })
    }
}

struct ImplStreamItemAstExtractor<'a> {
    ns: &'a Ident,
    method: &'a Ident,
    args: &'a [&'a Box<Type>],
}

impl<'a> ToTokens for ImplStreamItemAstExtractor<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let ns = &self.ns;
        let args = self.args;
        let method = self.method;

        let output = quote! {
            {
                struct Helper<I: #zod::core::ResponseType, S: #zod::__private::futures::Stream<Item = I> + 'static> {
                    _inner: &'static dyn Fn(&mut #ns, #(#args),*) -> S,
                }

                impl<I: #zod::core::ResponseType, S: #zod::__private::futures::Stream<Item = I> + 'static> Helper<I, S> {
                    const fn get(&self) -> #zod::core::ast::Definition {
                        <I as #zod::core::ResponseType>::AST
                    }
                }

                Helper {
                    _inner: &#ns::#method,
                }
            }
        };

        tokens.extend(output)
    }
}
