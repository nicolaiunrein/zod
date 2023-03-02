use darling::{ast::Data, FromDeriveInput};
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;

mod args;
mod backend_impl;
mod rpc_impl;

#[proc_macro_error]
#[proc_macro_derive(Backend, attributes(rpc))]
pub fn backend(input: TokenStream) -> TokenStream {
    let parsed = match syn::parse(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let input = match args::BackendInput::from_derive_input(&parsed) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let expanded = match input.data.clone() {
        Data::Enum(_) => unreachable!(),
        Data::Struct(e) => backend_impl::expand(input, e),
    };
    expanded.into()
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn rpc(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let orig = proc_macro2::TokenStream::from(input.clone());

    let ast = syn::parse_macro_input!(input as syn::ItemImpl);
    let extra = rpc_impl::expand(args::RpcInput::from_ast(ast));

    let output = quote! {
        #orig

        #extra
    };

    output.into()
}
