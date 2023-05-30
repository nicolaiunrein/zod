use proc_macro::TokenStream;
use zod_core::Kind;

#[proc_macro_derive(Zod, attributes(zod))]
pub fn zod_io(input: TokenStream) -> TokenStream {
    let mut input_impl = zod_core::derive_internals::impl_zod(Kind::Input, input.clone().into());
    let output_impl = zod_core::derive_internals::impl_zod(Kind::Output, input.into());
    input_impl.extend(output_impl);
    input_impl.into()
}

#[proc_macro_derive(ZodInputOnly, attributes(zod))]
pub fn zod_input(input: TokenStream) -> TokenStream {
    zod_core::derive_internals::impl_zod(Kind::Input, input.into()).into()
}

#[proc_macro_derive(ZodOutputOnly, attributes(zod))]
pub fn zod_output(input: TokenStream) -> TokenStream {
    zod_core::derive_internals::impl_zod(Kind::Output, input.into()).into()
}

#[cfg(test)]
mod test {
    #[test]
    fn ui_fail() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/fail_*.rs");
    }
}
