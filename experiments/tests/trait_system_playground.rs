//! # The trait system
//! This is quite tricky because we need to think on different levels:
//! - Generic Level:
//!    The monomorphized Type *does not* matter:
//!    ie. Generic<u8> === Generic<u16>
//!    For all generic arguments it always gives the same export.
//!
//!    InputOnly/OutputOnly types should also do this
//!
//! - Concrete Lvel:
//!   The monomorphized Type *does* matter:
//!   ie. Generic<u8> != Generic<u16>
//!   It yields different results for different generic arguments
//!   
use experiments::{
    types::{Role, ZodExport, ZodNamedField, ZodNumber, ZodObject, ZodType, ZodTypeInner},
    Namespace, Reference,
};

trait Type {
    const NAME: &'static str;
    const ARGS_NAMES: &'static [&'static str] = &[];
    const ROLE: Role;
    type Namespace: Namespace;

    fn value() -> ZodType;

    fn args() -> Vec<Reference> {
        Vec::new()
    }

    fn get_ref() -> Reference {
        Reference {
            ns: String::from(Self::Namespace::NAME),
            name: String::from(Self::NAME),
            role: Self::ROLE,
            args: Self::args().into_iter().map(|r| r.into()).collect(),
        }
    }
}

/// mark a Type as Input
/// The generic is there to avoid a warning
/// `https://github.com/rust-lang/rust/issues/48214`
trait Input<T = ()> {}

/// mark a Type as Output
/// The generic is there to avoid a warning
/// `https://github.com/rust-lang/rust/issues/48214`
trait Output<T = ()> {}

trait ExportExt {
    fn make_export() -> ZodExport;
}

impl<T> ExportExt for T
where
    T: Type,
{
    fn make_export() -> ZodExport {
        let name = T::NAME;
        let ns = T::Namespace::NAME;
        let value = T::value();
        let args = T::ARGS_NAMES;

        ZodExport::builder()
            .name(name)
            .ns(ns)
            .context(T::ROLE)
            .value(value)
            .args(args)
            .build()
    }
}
//
#[cfg(test)]
mod test {
    use experiments::types::{Ts, Zod};
    use pretty_assertions::assert_eq;

    use super::*;

    pub struct Ns;
    pub struct Rs;

    impl Namespace for Ns {
        const NAME: &'static str = "MyNs";
    }

    impl Namespace for Rs {
        const NAME: &'static str = "Rs";
    }

    pub struct Normal {
        pub value: u8,
    }

    pub struct Generic<T> {
        pub inner: T,
    }

    impl Type for u8 {
        const NAME: &'static str = "U8";
        const ROLE: Role = Role::Io;
        type Namespace = Rs;
        fn value() -> ZodType {
            ZodType::builder().inner(ZodNumber).build()
        }
    }

    impl Input for u8 {}
    impl Output for u8 {}

    impl Type for Normal {
        const ROLE: Role = Role::Io;
        type Namespace = Ns;
        const NAME: &'static str = "Normal";
        fn value() -> ZodType {
            ZodType::builder()
                .inner(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("value")
                            .value(u8::get_ref())
                            .build()])
                        .build(),
                )
                .build()
        }
    }

    impl Input for Normal {}
    impl Output for Normal {}

    impl<T> Type for Generic<T>
    where
        T: Type,
    {
        type Namespace = Ns;
        const ROLE: Role = Role::Io;
        const NAME: &'static str = "Generic";
        const ARGS_NAMES: &'static [&'static str] = &["T"];

        fn args() -> Vec<Reference> {
            vec![T::get_ref()]
        }

        fn value() -> ZodType {
            ZodType::builder()
                .inner(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("value")
                            .value(ZodTypeInner::Generic(String::from(Self::ARGS_NAMES[0])))
                            .build()])
                        .build(),
                )
                .build()
        }
    }

    impl<T> Input for Generic<T> where T: Input {}
    impl<T> Output for Generic<T> where T: Output {}

    struct InputOnlyStruct;

    impl Type for InputOnlyStruct {
        type Namespace = Ns;
        const ROLE: Role = Role::InputOnly;
        const NAME: &'static str = "InputOnlyStruct";

        fn value() -> ZodType {
            ZodType::builder()
                .inner(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("value")
                            .value(u8::get_ref())
                            .build()])
                        .build(),
                )
                .build()
        }
    }

    impl Input for InputOnlyStruct {}

    struct Nested {
        inner: Generic<InputOnlyStruct>,
    }

    impl Input for Nested {}
    impl<T> Output<T> for Nested where Generic<InputOnlyStruct>: Output<T> {}

    impl Type for Nested {
        type Namespace = Ns;
        const ROLE: Role = Role::Io;
        const NAME: &'static str = "Nested";

        fn value() -> ZodType {
            ZodType::builder()
                .inner(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("inner")
                            .value(<Generic<InputOnlyStruct>>::get_ref())
                            .build()])
                        .build(),
                )
                .build()
        }
    }

    #[test]
    fn exports_ok() {
        let normal_export = Normal::make_export();
        let u8_export = u8::make_export();
        let input_only_export = InputOnlyStruct::make_export();
        let generic_export = Generic::<u8>::make_export();
        let generic_input_only_export = Generic::<InputOnlyStruct>::make_export();

        assert_eq!(
            Zod(&normal_export).to_string(),
            "export const Normal = z.object({ value: Rs.io.U8 });"
        );

        assert_eq!(normal_export.context, Role::Io);

        assert_eq!(Zod(&u8_export).to_string(), "export const U8 = z.number();");

        assert_eq!(u8_export.context, Role::Io);

        assert_eq!(
            Zod(&input_only_export).to_string(),
            "export const InputOnlyStruct = z.object({ value: Rs.io.U8 });"
        );

        assert_eq!(input_only_export.context, Role::InputOnly);

        assert_eq!(
            Zod(&generic_export).to_string(),
            "export const Generic = (T: z.ZodTypeAny) => z.object({ value: T });"
        );

        assert_eq!(generic_export.context, Role::Io);

        assert_eq!(
            Zod(&generic_input_only_export).to_string(),
            "export const Generic = (T: z.ZodTypeAny) => z.object({ value: T });"
        );

        println!("{:#?}", Nested::make_export());

        assert_eq!(
            Ts(&Nested::make_export()).to_string(),
            "export interface Nested { inner: MyNs.io.Generic<MyNs.input.InputOnlyStruct> }"
        );

        assert_eq!(
            Zod(&Nested::make_export()).to_string(),
            "export const Nested = z.object({ inner: MyNs.io.Generic(MyNs.input.InputOnlyStruct) });"
        );
    }

    #[test]
    fn rpc_ok() {
        fn is_input<T>()
        where
            T: Input,
        {
        }

        fn is_output<T>()
        where
            T: Output,
        {
        }

        is_input::<Generic<InputOnlyStruct>>();
        is_input::<InputOnlyStruct>();
        // is_output::<Generic<InputOnlyStruct>>(); // should fail
        is_output::<Generic<u8>>();
        // is_output::<Nested>(); //should fail
    }
}
