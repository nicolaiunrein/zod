use darling::{ast::Data, FromDeriveInput};
use proc_macro::TokenStream;

mod args;
mod enum_impl;
mod struct_impl;

#[proc_macro_derive(zod, attributes(zod))]
pub fn zod(input: TokenStream) -> TokenStream {
    let parsed = match syn::parse(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };
    let input = match args::Input::from_derive_input(&parsed) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    let expanded = match input.data.clone() {
        Data::Enum(e) => enum_impl::expand(input, e),
        Data::Struct(e) => struct_impl::expand(input, e),
    };
    expanded.into()
}
