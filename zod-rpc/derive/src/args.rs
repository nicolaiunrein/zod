use darling::{ast::Data, FromDeriveInput, FromField};
use proc_macro2::Span;
use proc_macro_error::abort_call_site;
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
    pub arg_types: Vec<Box<Type>>,
    pub kind: RpcItemKind,
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

        let arg_types = sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                syn::FnArg::Receiver(_) => None,
                syn::FnArg::Typed(t) => Some(t.ty.clone()),
            })
            .collect();

        Self {
            ident,
            arg_types,
            kind,
        }
    }
}

pub enum RpcItemKind {
    Method,
    Stream,
}
