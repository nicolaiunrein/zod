use darling::ToTokens;
use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::{quote, quote_spanned};
use syn::{parse_quote, Ident, ImplItem, ImplItemMethod, ItemImpl, Type};

use crate::error::Error;
use crate::utils::get_zod;

pub(crate) struct RpcInput {
    ident: syn::Ident,
    items: Vec<RpcItem>,
}

pub(crate) struct RpcItem {
    ident: syn::Ident,
    method_args: Vec<RpcArg>,
    kind: RpcItemKind,
}

pub(crate) struct RpcArg {
    name: String,
    ty: Box<Type>,
}

pub(crate) enum RpcItemKind {
    Method(Box<Type>),
    Stream(Box<Type>),
}

impl RpcInput {
    fn enum_variants(&self) -> Vec<TokenStream> {
        self
            .items
            .iter()
            .map(|item| {
                let field_ident = &item.ident;
                let types = item.method_args.iter().map(|arg| &arg.ty);

                let args = if item.method_args.is_empty() {
                    // In serde empty sequences cannot be deserialized to the unit type at the moment.
                    // See: https://github.com/serde-rs/serde/issues/2340
                    quote!([(); 0]) 
                } else {
                    quote!((#(#types,)*))
                };

                quote!(#field_ident { args: #args })
            })
            .collect()
    }

    fn input_types(&self) -> Vec<&Box<syn::Type>> {
        self
            .items
            .iter()
            .flat_map(|item| item.method_args.iter().map(|arg| &arg.ty))
            .collect()

    }

    fn rpc_requests(&self) -> Vec<TokenStream> {
        let ident = &self.ident;
        let zod =get_zod();

        self.items.iter().map(|item| {
            let name = item.ident.to_string();

            let args = item.method_args.iter().map(|arg| {
                let ty = &arg.ty;
                let name = &arg.name;
                quote!(#zod::core::ast::NamedField::new_req::<#ty>(#name))
            });

            match &item.kind {
                RpcItemKind::Method(output) => {
                    quote_spanned! { ident.span() =>
                        #zod::core::ast::rpc::RpcRequest {
                            path: #zod::core::ast::Path::new::<#ident>(#name),
                            args: &[#(#args),*],
                            output: #zod::core::ast::Ref::new_res::<#output>(),
                            kind: #zod::core::ast::rpc::RpcRequestKind::Method,
                        }
                    }
                }
                RpcItemKind::Stream(_) => {

                    let method_ident = &item.ident;
                    let ns_ident = &self.ident;
                    let todos = item.method_args.iter().map(|_| quote!(todo!()));

                    quote_spanned! { item.ident.span() =>
                        #zod::core::ast::rpc::RpcRequest {
                            path: #zod::core::ast::Path::new::<#ident>(#name),
                            args: &[#(#args),*],
                            output: #zod::core::ast::Ref::new_stream_res(&|| #ns_ident::#method_ident(todo!(), #(#todos),*)),
                            kind: #zod::core::ast::rpc::RpcRequestKind::Stream,
                        }
                    }
                }
            }
        }).collect()
    }

    fn match_arms(&self) -> Vec<TokenStream> {
        let zod = get_zod();

        self.items.iter().map(|item| {
            let ident = &item.ident;
            let args = item.method_args.iter().enumerate().map(|(i, _)| {
                let index = syn::Index::from(i);
                quote!(args.#index)
            });

            match &item.kind {
                RpcItemKind::Method(_) => {
                    quote_spanned! { ident.span() => 
                            #[allow(unused_variables)]
                            Self::Req:: #ident { args } => {
                            let res = self.#ident(#(#args),*).await;
                            let _ = sender.unbounded_send(#zod::core::rpc::Response::method(id, res));
                            None
                        }
                    }
                }
                RpcItemKind::Stream(_) => {
                    quote_spanned! { ident.span() => 
                        #[allow(unused_variables)]
                        Self::Req:: #ident { args } => {
                            let stream = self.#ident(#(#args),*);
                            let jh = tokio::spawn(async move {
                                #zod::__private::futures::pin_mut!(stream);
                                while let Some(evt) = #zod::__private::futures::StreamExt::next(&mut stream).await {
                                    let _ = sender.unbounded_send(#zod::core::rpc::Response::stream(id, evt));
                                }
                            });

                            Some(jh.into())
                        }
                    }
                }
            }
        }).collect()

    }

    fn output_types(&self) -> Vec<TokenStream> {
        let zod = get_zod();

        self.items.iter().map(|item| {
            match &item.kind {
                RpcItemKind::Method(output) => quote!(<#output as #zod::core::ResponseTypeVisitor>::register(ctx)),
                RpcItemKind::Stream(_output) => {
                    let ns_ident = &self.ident;
                    let method_ident = &item.ident;
                    let todos = item.method_args.iter().map(|_| quote!(todo!()));

                    quote_spanned!{ item.ident.span() => 
                        #[allow(unreachable_code)]
                        ctx.add_stream_output(|| #ns_ident::#method_ident(todo!(), #(#todos),*))
                    }
                }                                                                                  
            }
        }).collect()

    }
}

impl ToTokens for RpcInput {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let ident = &self.ident;
        let enum_variants = self.enum_variants();
        let input_types = self.input_types();
        let rpc_requests = self.rpc_requests();
        let match_arms = self.match_arms();
        let output_types = self.output_types();

        let req_name = quote::format_ident!("{}Req", ident);

        let output = quote_spanned! {
            ident.span() =>
            const _: () = {
                #[derive(#zod::__private::serde::Deserialize)]
                #[serde(tag = "method")]
                #[allow(non_camel_case_types)]
                #[allow(non_snake_case)]
                #[allow(non_upper_case_globals)]
                #[doc(hidden)]
                pub enum #req_name {
                    #(#enum_variants),*
                }

                impl #zod::core::RequestTypeVisitor for #req_name {
                    #[allow(unused_variables, unused_mut)]
                    fn register(ctx: &mut #zod::core::DependencyMap)
                    where
                        Self: 'static,
                    {
                        #(<#input_types as #zod::core::RequestTypeVisitor>::register(ctx));*
                    }
                }

                impl #zod::core::ResponseTypeVisitor for #req_name {
                    #[allow(unused_variables, unused_mut)]
                    fn register(ctx: &mut #zod::core::DependencyMap)
                    where
                        Self: 'static,
                    {

                        #(#output_types);*
                    }
                }

                #[#zod::__private::async_trait::async_trait]
                impl #zod::core::rpc::RpcNamespace for #ident {

                    #[allow(unreachable_code)]
                    const AST: &'static [#zod::core::ast::rpc::RpcRequest] = &[
                        #(#rpc_requests),*
                    ];

                    type Req = #req_name;

                    async fn process(
                        &mut self,
                        req: Self::Req,
                        sender: #zod::core::rpc::ResponseSender,
                        id: usize,
                    ) -> Option<#zod::core::rpc::server::StreamHandle> {
                        match req {
                            #(#match_arms),*
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
        let items = 
            input
                .items
                .into_iter()
                .filter_map(|item| match item {
                    ImplItem::Method(method) => Some(method),
                    _ => None,
                })
                .map(RpcItem::try_from)
                .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            ident,
            items
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
                (None, Some((ampersand, _))) => return Err(Error::shared_self(ampersand.span).into()),
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

