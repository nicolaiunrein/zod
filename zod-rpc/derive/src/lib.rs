use darling::{ast::Data, FromDeriveInput};
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod args;
mod backend_impl;

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

