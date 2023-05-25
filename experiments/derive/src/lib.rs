use proc_macro::TokenStream;
use zod_core::types::Role;

#[proc_macro_derive(Zod, attributes(zod))]
pub fn zod_io(input: TokenStream) -> TokenStream {
    zod_core::derive_internals::impl_zod(Role::Io, input.into()).into()
}

#[proc_macro_derive(ZodInputOnly, attributes(zod))]
pub fn zod_input(input: TokenStream) -> TokenStream {
    zod_core::derive_internals::impl_zod(Role::InputOnly, input.into()).into()
}

#[proc_macro_derive(ZodOutputOnly, attributes(zod))]
pub fn zod_output(input: TokenStream) -> TokenStream {
    zod_core::derive_internals::impl_zod(Role::OutputOnly, input.into()).into()
}

#[cfg(test)]
mod test {
    #[test]
    fn ui_fail() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/fail_*.rs");
    }
}
