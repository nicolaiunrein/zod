use darling::{ast::Data, FromDeriveInput};
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod args;
mod impl_enum;
mod impl_struct;

#[proc_macro_error]
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
        Data::Enum(e) => impl_enum::expand(input, e),
        Data::Struct(e) => impl_struct::expand(input, e),
    };
    expanded.into()
}
