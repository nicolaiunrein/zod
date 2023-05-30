#![allow(dead_code)]
use pretty_assertions::assert_eq;
use zod_core::{types::ZodExport, Kind, Type};
use zod_derive_experiments::Zod;

struct Ns;
impl zod_core::Namespace for Ns {
    const NAME: &'static str = "Custom_Ns";
}

#[derive(Zod)]
#[zod(namespace = "Ns")]
struct X<T: Type<Kind::Input> + Type<Kind::Output>> {
    inner: T,
}

#[test]
fn generic_ok() {
    let input: ZodExport<Kind::Input> = X::<String>::export();
    assert_eq!(
        zod_core::types::Zod(&input).to_string(),
        "export const X = (T: z.ZodTypeAny) => z.object({ inner: T });"
    )
}
