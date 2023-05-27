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

pub trait AsInputRole {
    const ROLE: Role;
}
pub trait AsOutputRole {
    const ROLE: Role;
}

mod marker {
    use super::AsInputRole;
    use super::AsOutputRole;
    use experiments::types::Role;

    pub struct InputOnly;
    pub struct OutputOnly;
    pub struct Both;

    impl AsInputRole for InputOnly {
        const ROLE: Role = Role::InputOnly;
    }

    impl AsOutputRole for OutputOnly {
        const ROLE: Role = Role::OutputOnly;
    }

    impl AsInputRole for Both {
        const ROLE: Role = Role::Io;
    }

    impl AsOutputRole for Both {
        const ROLE: Role = Role::Io;
    }
}

trait Type {
    const NAME: &'static str;
    const ARGS_NAMES: &'static [&'static str] = &[];
    type Namespace: Namespace;
    type Role;
}

trait InputType
where
    Self: Type,
    Self::Role: AsInputRole,
{
    fn input_value() -> ZodType;
    fn get_input_args() -> Vec<ZodType>;
    fn get_input_ref() -> Reference {
        Reference {
            ns: String::from(Self::Namespace::NAME),
            name: String::from(Self::NAME),
            role: <Self as Type>::Role::ROLE,
            args: Self::get_input_args(),
        }
    }
}
trait OutputType
where
    Self: Type,
    Self::Role: AsOutputRole,
{
    fn output_value() -> ZodType;
    fn get_output_args() -> Vec<ZodType>;
    fn get_output_ref() -> Reference {
        Reference {
            ns: String::from(Self::Namespace::NAME),
            name: String::from(Self::NAME),
            role: <Self as Type>::Role::ROLE,
            args: Self::get_output_args(),
        }
    }
}

fn make_input_export<T>() -> ZodExport
where
    T: Type,
    T: InputType,
    T::Role: AsInputRole,
{
    let name = T::NAME;
    let ns = T::Namespace::NAME;
    let value = T::input_value();
    let args = T::ARGS_NAMES;

    ZodExport::builder()
        .name(name)
        .ns(ns)
        .context(T::get_input_ref().role)
        .value(value)
        .args(args)
        .build()
}

fn make_output_export<T>() -> ZodExport
where
    T: Type,
    T: OutputType,
    T::Role: AsOutputRole,
{
    let name = T::NAME;
    let ns = T::Namespace::NAME;
    let value = T::output_value();
    let args = T::ARGS_NAMES;

    ZodExport::builder()
        .name(name)
        .ns(ns)
        .context(T::get_output_ref().role)
        .value(value)
        .args(args)
        .build()
}

#[cfg(test)]
mod test {
    use experiments::types::Zod;
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
        type Role = marker::Both;
        const NAME: &'static str = "U8";
        type Namespace = Rs;
    }

    impl InputType for u8 {
        fn input_value() -> ZodType {
            ZodType::builder().inner(ZodNumber).build()
        }

        fn get_input_args() -> Vec<ZodType> {
            Vec::new()
        }
    }

    impl OutputType for u8 {
        fn output_value() -> ZodType {
            ZodType::builder().inner(ZodNumber).build()
        }

        fn get_output_args() -> Vec<ZodType> {
            Vec::new()
        }
    }

    impl Type for Normal {
        type Role = marker::Both;
        type Namespace = Ns;
        const NAME: &'static str = "Normal";
    }

    impl InputType for Normal {
        fn input_value() -> ZodType {
            ZodType::builder()
                .inner(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("value")
                            .value(u8::get_input_ref())
                            .build()])
                        .build(),
                )
                .build()
        }

        fn get_input_args() -> Vec<ZodType> {
            Vec::new()
        }
    }

    impl OutputType for Normal {
        fn output_value() -> ZodType {
            ZodType::builder()
                .inner(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("value")
                            .value(u8::get_output_ref())
                            .build()])
                        .build(),
                )
                .build()
        }

        fn get_output_args() -> Vec<ZodType> {
            Vec::new()
        }
    }

    impl<T> Type for Generic<T>
    where
        T: Type,
    {
        type Namespace = Ns;
        type Role = marker::Both;
        const NAME: &'static str = "Generic";
        const ARGS_NAMES: &'static [&'static str] = &["T"];
    }

    impl<T> InputType for Generic<T>
    where
        T: InputType,
        T::Role: AsInputRole,
    {
        fn input_value() -> ZodType {
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
        fn get_input_args() -> Vec<ZodType> {
            vec![T::input_value()]
        }
    }
    impl<T> OutputType for Generic<T>
    where
        T: OutputType,
        T::Role: AsOutputRole,
    {
        fn output_value() -> ZodType {
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

        fn get_output_args() -> Vec<ZodType> {
            vec![T::output_value()]
        }
    }

    struct InputOnly;

    impl Type for InputOnly {
        type Namespace = Ns;
        type Role = marker::InputOnly;
        const NAME: &'static str = "InputOnly";
    }

    impl InputType for InputOnly {
        fn input_value() -> ZodType {
            ZodType::builder()
                .inner(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("value")
                            .value(u8::get_input_ref())
                            .build()])
                        .build(),
                )
                .build()
        }

        fn get_input_args() -> Vec<ZodType> {
            Vec::new()
        }
    }

    #[test]
    fn exports_ok() {
        let normal_export = make_input_export::<Normal>();
        let u8_export = make_input_export::<u8>();
        let input_only_export = make_input_export::<InputOnly>();
        let generic_export = make_input_export::<Generic<u8>>();
        let generic_input_only_export = make_input_export::<Generic<InputOnly>>();

        assert_eq!(
            Zod(&normal_export).to_string(),
            "export const Normal = z.object({ value: Rs.io.U8 });"
        );

        assert_eq!(normal_export.context, Role::Io);

        assert_eq!(Zod(&u8_export).to_string(), "export const U8 = z.number();");

        assert_eq!(u8_export.context, Role::Io);

        assert_eq!(
            Zod(&input_only_export).to_string(),
            "export const InputOnly = z.object({ value: Rs.io.U8 });"
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
    }

    #[test]
    fn rpc_ok() {
        fn make_rpc_arg<T>()
        where
            T: Type,
            T::Role: AsInputRole,
        {
        }

        fn make_rpc_response<T>()
        where
            T: Type,
            T::Role: AsOutputRole,
        {
        }

        make_rpc_arg::<Generic<InputOnly>>();
        make_rpc_arg::<InputOnly>();
        make_rpc_response::<Generic<InputOnly>>(); // should fail
        make_rpc_response::<Generic<u8>>();
    }
}
