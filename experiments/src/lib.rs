/// The impl is split between Argument types and Response types.
/// The Traits are the same. Find a way to uniform them.
mod build_ins;
mod const_str;
pub mod types;
mod utils;

#[cfg(test)]
mod test_utils;

use std::{collections::HashSet, fmt::Display};

use quote::{quote, ToTokens};
use typed_builder::TypedBuilder;
use types::{Ts, Zod, ZodExport, ZodType, ZodTypeInner};

pub trait InputType {
    fn get_input_ref() -> ZodType;
    fn visit_de_exports(_set: &mut HashSet<ZodExport>);

    fn collect_de_exports() -> HashSet<ZodExport> {
        let mut set = HashSet::new();
        Self::visit_de_exports(&mut set);
        set
    }
}

pub trait OutputType {
    fn get_output_ref() -> ZodType;
    fn visit_ser_exports(_set: &mut HashSet<ZodExport>);

    fn collect_ser_exports() -> HashSet<ZodExport> {
        let mut set = HashSet::new();
        Self::visit_ser_exports(&mut set);
        set
    }
}

pub trait IoType {
    fn get_ref() -> ZodType;
    fn visit_exports(_set: &mut HashSet<ZodExport>);

    fn collect_all_exports() -> HashSet<ZodExport> {
        let mut set = HashSet::new();
        Self::visit_exports(&mut set);
        set
    }
}

impl<T> OutputType for T
where
    T: IoType,
{
    fn get_output_ref() -> ZodType {
        Self::get_ref()
    }

    fn visit_ser_exports(set: &mut HashSet<ZodExport>) {
        Self::visit_exports(set)
    }
}

impl<T> InputType for T
where
    T: IoType,
{
    fn get_input_ref() -> ZodType {
        Self::get_ref()
    }

    fn visit_de_exports(set: &mut HashSet<ZodExport>) {
        Self::visit_exports(set)
    }
}

impl<const C: char, T: const_str::Chain> IoType for const_str::ConstStr<C, T> {
    fn get_ref() -> ZodType {
        ZodType::builder()
            .inner(ZodTypeInner::Generic(Self::value().to_string()))
            .build()
    }

    fn visit_exports(_set: &mut HashSet<ZodExport>) {}
}

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct Reference {
    #[builder(setter(into))]
    pub name: String,

    #[builder(setter(into))]
    pub ns: String,

    #[builder(default)]
    pub args: Vec<ZodType>,
}

impl ToTokens for Reference {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use crate::utils::crate_name;
        let name = &self.name;
        let args = &self.args;
        let ns = &self.ns;

        tokens.extend(quote!(#crate_name::Reference {
            ns: String::from(#ns),
            name: String::from(#name),
            args: vec![#(#args),*]
        }))
    }
}

impl From<Reference> for ZodTypeInner {
    fn from(value: Reference) -> Self {
        ZodTypeInner::Reference(value)
    }
}

impl<'a> Display for Ts<'a, Reference> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}.{}.{}",
            self.0.ns,
            self.context(),
            self.0.name
        ))?;
        if !self.0.args.is_empty() {
            let args = self
                .0
                .args
                .iter()
                .map(|arg| Ts(arg, self.1))
                .collect::<Vec<_>>();

            f.write_fmt(format_args!("<{}>", utils::Separated(", ", &args)))?;
        }
        Ok(())
    }
}

impl<'a> Display for Zod<'a, Reference> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}.{}.{}",
            self.0.ns,
            self.context(),
            self.0.name
        ))?;
        if !self.0.args.is_empty() {
            self.0.name.fmt(f)?;
            let args = self
                .0
                .args
                .iter()
                .map(|arg| Zod(arg, self.1))
                .collect::<Vec<_>>();
            f.write_fmt(format_args!("({})", utils::Separated(", ", &args)))?;
        }
        Ok(())
    }
}

// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
//
#[cfg(test)]
mod test {

    #![allow(dead_code)]
    use super::*;

    use pretty_assertions::assert_eq;

    use types::*;

    struct Generic<T> {
        inner: T,
    }

    impl<T> OutputType for Generic<T>
    where
        T: OutputType,
    {
        fn get_output_ref() -> ZodType {
            Reference::builder()
                .name("Generic")
                .ns("Ns")
                .args(vec![T::get_output_ref()])
                .build()
                .into()
        }

        fn visit_ser_exports(set: &mut HashSet<ZodExport>) {
            let export = ZodExport::builder()
                .ns("Ns")
                .name("Generic")
                .args(&["T"])
                .value(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("inner")
                            .value(ZodTypeInner::Generic(String::from("T")))
                            .build()])
                        .build(),
                )
                .build();

            set.insert(export);
            T::visit_ser_exports(set)
        }
    }

    impl<T> InputType for Generic<T>
    where
        T: InputType,
    {
        fn get_input_ref() -> ZodType {
            Reference::builder()
                .ns("Ns")
                .name("Generic")
                .args(vec![T::get_input_ref()])
                .build()
                .into()
        }

        fn visit_de_exports(set: &mut HashSet<ZodExport>) {
            let export = ZodExport::builder()
                .ns("Ns")
                .name("Generic")
                .args(&["T"])
                .value(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("inner")
                            .value(ZodTypeInner::Generic(String::from("T")))
                            .build()])
                        .build(),
                )
                .build();

            set.insert(export);
            T::visit_de_exports(set)
        }
    }

    struct Transparent;

    impl OutputType for Transparent {
        fn get_output_ref() -> ZodType {
            <String as OutputType>::get_output_ref()
        }
        fn visit_ser_exports(set: &mut HashSet<ZodExport>) {
            String::visit_ser_exports(set);
        }
    }

    impl InputType for Transparent {
        fn get_input_ref() -> ZodType {
            <u8 as InputType>::get_input_ref()
        }
        fn visit_de_exports(set: &mut HashSet<ZodExport>) {
            u8::visit_de_exports(set);
        }
    }

    struct Nested<T> {
        inner: Generic<T>,
    }

    impl<T: OutputType> OutputType for Nested<T> {
        fn get_output_ref() -> ZodType {
            Reference {
                name: String::from("Nested"),
                ns: String::from("Ns"),
                args: vec![T::get_output_ref()],
            }
            .into()
        }

        fn visit_ser_exports(set: &mut HashSet<ZodExport>) {
            let exp = ZodExport::builder()
                .ns("Ns")
                .name("Nested")
                .args(&["T"])
                .value(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("inner")
                            .value(Generic::<crate::const_str!('T')>::get_output_ref())
                            .build()])
                        .build(),
                )
                .build();

            set.insert(exp.into());

            T::visit_ser_exports(set)
        }
    }

    impl<T: InputType> InputType for Nested<T> {
        fn get_input_ref() -> ZodType {
            Reference {
                ns: String::from("Ns"),
                name: String::from("Nested"),
                args: vec![T::get_input_ref()],
            }
            .into()
        }
        fn visit_de_exports(set: &mut HashSet<ZodExport>) {
            let exp = ZodExport::builder()
                .ns("Ns")
                .name("Nested")
                .args(&["T"])
                .value(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("inner")
                            .value(Generic::<crate::const_str!('T')>::get_input_ref())
                            .build()])
                        .build(),
                )
                .build();

            set.insert(exp.into());

            T::visit_de_exports(set)
        }
    }

    struct SerOnly;

    impl OutputType for SerOnly {
        fn get_output_ref() -> ZodType {
            Reference {
                ns: String::from("Ns"),
                name: String::from("SerOnly"),
                args: Vec::new(),
            }
            .into()
        }
        fn visit_ser_exports(_set: &mut HashSet<ZodExport>) {}
    }

    #[test]
    fn inline_transparent_ok() {
        assert_eq!(
            Ts::io(&Transparent::get_output_ref()).to_string(),
            "Rs.io.String"
        );
        assert_eq!(
            Ts::io(&Transparent::get_input_ref()).to_string(),
            "Rs.io.U8"
        );
    }

    #[test]
    fn debug() {
        let u8_export = u8::collect_all_exports().into_iter().next().unwrap();
        let string_export = String::collect_all_exports().into_iter().next().unwrap();

        let generic_export = ZodExport::builder()
            .ns("Ns")
            .name("Generic")
            .args(&["T"])
            .value(
                ZodObject::builder()
                    .fields(vec![ZodNamedField::builder()
                        .name("inner")
                        .value(ZodTypeInner::Generic(String::from("T")))
                        .build()])
                    .build(),
            )
            .build();

        assert_eq!(
            Ts::io(&Generic::<Transparent>::get_output_ref()).to_string(),
            "Ns.io.Generic<Rs.io.String>"
        );

        assert_eq!(
            Ts::io(&Generic::<crate::const_str!('M', 'Y', '_', 'T')>::get_output_ref()).to_string(),
            "Ns.io.Generic<MY_T>"
        );

        assert_eq!(
            Ts::io(&Generic::<Transparent>::get_input_ref()).to_string(),
            "Ns.io.Generic<Rs.io.U8>"
        );

        assert_eq!(
            <Generic::<u8>>::collect_ser_exports(),
            [u8_export.clone(), generic_export.clone()]
                .into_iter()
                .collect()
        );

        assert_eq!(
            Transparent::collect_ser_exports(),
            [string_export.clone()].into_iter().collect()
        );

        assert_eq!(
            <Generic::<Transparent>>::collect_ser_exports(),
            [string_export.clone(), generic_export.clone()]
                .into_iter()
                .collect()
        );

        assert_eq!(
            <Generic::<SerOnly>>::collect_ser_exports(),
            [generic_export.clone()].into_iter().collect()
        );

        assert_eq!(
            <Generic::<SerOnly>>::get_output_ref(),
            Reference {
                ns: String::from("Ns"),
                name: String::from("Generic"),
                args: vec![Reference {
                    ns: String::from("Ns"),
                    name: String::from("SerOnly"),
                    args: vec![]
                }
                .into()]
            }
            .into()
        );
    }
}
