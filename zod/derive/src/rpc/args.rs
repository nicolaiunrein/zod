use darling::{ast::Data, FromDeriveInput, FromField};
use proc_macro2::Span;
use proc_macro_error::{abort, abort_call_site};
use quote::format_ident;
use syn::{parse_quote, Ident, ImplItem, ImplItemMethod, ItemImpl, Path, Type};

pub fn get_zod() -> Path {
    let name = get_crate_name();
    let ident = Ident::new(&name, Span::call_site());
    parse_quote!(::#ident)
}

pub fn get_private() -> Path {
    let zod = get_zod();
    parse_quote!(#zod::rpc::__private)
}

#[derive(FromDeriveInput)]
pub struct BackendInput {
    pub ident: syn::Ident,
    pub data: Data<darling::util::Ignored, BackendField>,
}

#[derive(FromField, Clone)]
pub struct BackendField {
    pub ident: Option<syn::Ident>,
    pub ty: Type,
}

fn get_crate_name() -> String {
    let found_crate = proc_macro_crate::crate_name("zod").unwrap_or_else(|err| {
        abort_call_site!("Error: {}", err);
    });

    match found_crate {
        proc_macro_crate::FoundCrate::Itself => String::from("zod"),
        proc_macro_crate::FoundCrate::Name(name) => name,
    }
}

pub struct RpcInput {
    pub ident: syn::Ident,
    pub items: Vec<RpcItem>,
}

impl RpcInput {
    pub fn from_ast(ast: ItemImpl) -> Self {
        let self_ty = ast.self_ty;
        let ident: Ident = parse_quote!(#self_ty);
        Self {
            ident,
            items: ast
                .items
                .into_iter()
                .filter_map(|item| match item {
                    ImplItem::Method(method) => Some(method),
                    _ => None,
                })
                .map(RpcItem::from_ast)
                .collect(),
        }
    }

    pub(crate) fn req_ident(&self) -> Ident {
        format_ident!("{}Req", self.ident)
    }
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

impl RpcItem {
    pub fn from_ast(ast: ImplItemMethod) -> Self {
        let sig = ast.sig;
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
                abort!(
                    ident.span(),
                    "zod: namespace methods must be async or return a stream"
                )
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
                    abort! {
                    lifetime.span(),
                    "zod: namespace methods are not allowed to have lifetimes"

                    }
                }
                (None, None) => abort!(
                    receiver.self_token.span,
                    "zod: expected `&mut self` got `self`.",
                ),
                (None, Some((and, _))) => {
                    abort!(and.span, "zod: expected `&mut self` got `&self`.",)
                }
                (Some(_), None) => abort!(
                    receiver.self_token.span,
                    "zod: expected `&mut self` got `mut self`.",
                ),
            }
        } else {
            abort!(
                ident.span(),
                "zod: namespace methods must have a self argument"
            );
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

        Self {
            ident,
            arg_types,
            kind,
            output,
        }
    }
}

pub enum RpcItemKind {
    Method,
    Stream,
}
