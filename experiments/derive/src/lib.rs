use proc_macro::TokenStream;

#[proc_macro_derive(Zod, attributes(zod))]
pub fn zod(input: TokenStream) -> TokenStream {
    zod_core::derive_internals::impl_zod(input.into()).into()
}

#[cfg(test)]
mod test {
    #[test]
    fn ui_fail() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/fail_*.rs");
    }
}
