mod backend;
mod config;
mod docs;
mod r#enum;
mod error;
mod field;
mod namespace;
mod node;
mod r#struct;
mod test_utils;
mod utils;

mod rpc;

use backend::BackendInput;
use darling::FromDeriveInput;
use namespace::Namespace;
use node::{Derive, ZodType};
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;

#[proc_macro_error]
#[proc_macro_derive(RequestType, attributes(zod))]
pub fn request(input: TokenStream) -> TokenStream {
    let parsed = match syn::parse(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let request_type = match ZodType::from_derive_input(&parsed, Derive::Request) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    quote!(#request_type).into()
}

#[proc_macro_error]
#[proc_macro_derive(ResponseType, attributes(zod))]
pub fn response(input: TokenStream) -> TokenStream {
    let parsed = match syn::parse(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let request_type = match ZodType::from_derive_input(&parsed, Derive::Response) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    quote!(#request_type).into()
}

#[proc_macro_error]
#[proc_macro_derive(Namespace, attributes(zod))]
pub fn ns(input: TokenStream) -> TokenStream {
    let parsed = match syn::parse(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let ns = match Namespace::from_derive_input(&parsed) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    quote!(#ns).into()
}

#[proc_macro_error]
#[proc_macro_derive(Backend, attributes(zod))]
pub fn backend(input: TokenStream) -> TokenStream {
    let parsed = match syn::parse(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let backend = match BackendInput::from_derive_input(&parsed) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    quote!(#backend).into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn rpc(_attrs: TokenStream, input: TokenStream) -> TokenStream {
    let orig = proc_macro2::TokenStream::from(input.clone());

    let ast = syn::parse_macro_input!(input as syn::ItemImpl);

    let input = match rpc::RpcInput::try_from(ast) {
        Ok(v) => v,
        Err(err) => {
            return syn::Error::from(err).into_compile_error().into();
        }
    };

    let output = quote! {
        #orig
        #input
    };

    output.into()
}
