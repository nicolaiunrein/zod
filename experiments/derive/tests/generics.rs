#![allow(dead_code)]
use pretty_assertions::assert_eq;
use zod_core::{types::ZodExport, Kind, Type};
use zod_derive_experiments::Zod;
use zod_derive_experiments::ZodInputOnly;

struct Ns;
impl zod_core::Namespace for Ns {
    const NAME: &'static str = "Custom_Ns";
}

#[derive(Zod)]
#[zod(namespace = "Ns")]
struct X<T> {
    inner: T,
}

#[derive(Zod)]
#[zod(namespace = "Ns")]
enum Enum<T> {
    A(String),
    B(T),
    // C(X<Other>),
}

#[derive(ZodInputOnly)]
#[zod(namespace = "Ns")]
struct Other {}

#[test]
fn generic_ok() {
    let input: ZodExport<Kind::Input> = <X<Other> as Type<Kind::Input>>::export();
    assert_eq!(
        zod_core::types::Zod(&input).to_string(),
        "export const X = (T: z.ZodTypeAny) => z.object({ inner: T });"
    )
}
