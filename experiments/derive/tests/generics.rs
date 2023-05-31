use pretty_assertions::assert_eq;
use zod_core::prelude::*;
use zod_derive_experiments::ZodInputOnly;

struct Ns;
impl zod_core::Namespace for Ns {
    const NAME: &'static str = "Ns";
}

trait SomeTrait {}
impl SomeTrait for String {}

#[test]
fn generic_without_bounds_ok() {
    #![allow(dead_code)]

    struct Ns;
    impl zod_core::Namespace for Ns {
        const NAME: &'static str = "Ns";
    }

    #[derive(ZodInputOnly)]
    #[zod(namespace = "Ns")]
    struct Generic<T> {
        inner: T,
    }

    #[derive(ZodInputOnly)]
    #[zod(namespace = "Ns")]
    struct Nested<T> {
        nested: Generic<T>,
    }

    assert_eq!(
        Generic::<String>::export().unwrap().as_zod().to_string(),
        "export const Generic = (T: z.ZodTypeAny) => z.object({ inner: T });"
    );

    assert_eq!(
        Nested::<String>::export().unwrap().as_zod().to_string(),
        "export const Nested = (T: z.ZodTypeAny) => z.object({ nested: Ns.input.Generic(T) });"
    );
}

#[test]
fn generic_with_bounds_ok() {
    #![allow(dead_code)]

    struct Ns;
    impl zod_core::Namespace for Ns {
        const NAME: &'static str = "Ns";
    }

    #[derive(ZodInputOnly)]
    #[zod(namespace = "Ns")]
    struct Generic<T: SomeTrait> {
        inner: T,
    }

    #[derive(ZodInputOnly)]
    #[zod(namespace = "Ns")]
    struct Nested<T: SomeTrait> {
        nested: Generic<T>,
    }

    assert_eq!(
        Generic::<String>::export().unwrap().as_zod().to_string(),
        "export const Generic = (T: z.ZodTypeAny) => z.object({ inner: T });"
    );

    // Nested has bounds and cannot be filled in by generics hence it does not get exported.
    assert_eq!(Nested::<String>::export(), None);

    // it is not referenced but inlined as is.
    assert_eq!(
        Nested::<String>::inline().as_zod().to_string(),
        "z.object({ nested: Ns.input.Generic(Rs.input.String) })"
    );
}
