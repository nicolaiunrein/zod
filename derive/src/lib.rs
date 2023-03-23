mod docs;
mod r#enum;
mod field;
mod node;
mod r#struct;
mod test_utils;
mod utils;

use darling::FromDeriveInput;
use docs::RustDocs;
use node::ZodNode;
use proc_macro::TokenStream;
// use proc_macro_error::proc_macro_error;

// #[proc_macro_error]
#[proc_macro_derive(Node, attributes(zod))]
pub fn node(input: TokenStream) -> TokenStream {
    let parsed = match syn::parse(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let cx = serde_derive_internals::Ctxt::new();

    let container = serde_derive_internals::ast::Container::from_ast(
        &cx,
        &parsed,
        serde_derive_internals::Derive::Deserialize,
    )
    .unwrap();

    cx.check().unwrap();

    let docs = match RustDocs::from_attrs(&parsed.attrs) {
        Ok(docs) => docs,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };

    let node = match ZodNode::from_derive_input(&parsed) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };

    node.expand(&container, &docs).into()
}
