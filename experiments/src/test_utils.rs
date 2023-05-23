use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;

pub(crate) fn formatted(input: impl ToTokens) -> String {
    let file = quote!(fn test() {#input});
    let syntax_tree = syn::parse_file(&file.to_string()).unwrap();
    prettyplease::unparse(&syntax_tree)
}

pub(crate) fn expand_zod(input: impl ToTokens) -> TokenStream {
    quote!(#input)
        .to_string()
        .replace("crate", "::zod::core")
        .parse::<TokenStream>()
        .unwrap()
}
