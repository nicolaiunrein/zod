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
use types::{Ts, Zod, ZodExport, ZodTypeInner};

pub trait InputType {
    fn get_input_ref() -> Reference;
    fn visit_de_exports(_set: &mut HashSet<ZodExport>);

    fn collect_de_exports() -> HashSet<ZodExport> {
        let mut set = HashSet::new();
        Self::visit_de_exports(&mut set);
        set
    }
}

pub trait OutputType {
    fn get_output_ref() -> Reference;
    fn visit_ser_exports(_set: &mut HashSet<ZodExport>);

    fn collect_ser_exports() -> HashSet<ZodExport> {
        let mut set = HashSet::new();
        Self::visit_ser_exports(&mut set);
        set
    }
}

pub trait IoType {
    fn get_ref() -> Reference;
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
    fn get_output_ref() -> Reference {
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
    fn get_input_ref() -> Reference {
        Self::get_ref()
    }

    fn visit_de_exports(set: &mut HashSet<ZodExport>) {
        Self::visit_exports(set)
    }
}

impl<const C: char, T: const_str::Chain> IoType for const_str::ConstStr<C, T> {
    fn get_ref() -> Reference {
        Reference {
            ns: String::from("todo"),
            name: Self::value().to_string(),
            args: Vec::new(),
        }
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
    pub args: Vec<Reference>,
}

impl ToTokens for Reference {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        use crate::utils::crate_name;
        let name = &self.name;
        let args = &self.args;
        tokens.extend(quote!(#crate_name::Reference {
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
        if self.0.args.is_empty() {
            f.write_fmt(format_args!("{}", self.0.name))
        } else {
            self.0.name.fmt(f)?;
            let args = self.0.args.iter().map(Ts).collect::<Vec<_>>();

            f.write_fmt(format_args!("<{}>", utils::Separated(", ", &args)))?;
            Ok(())
        }
    }
}

impl<'a> Display for Zod<'a, Reference> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.args.is_empty() {
            f.write_fmt(format_args!("{}", self.0.name))
        } else {
            self.0.name.fmt(f)?;
            let args = self.0.args.iter().map(Zod).collect::<Vec<_>>();
            f.write_fmt(format_args!("({})", utils::Separated(", ", &args)))?;
            Ok(())
        }
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
        fn get_output_ref() -> Reference {
            Reference::builder()
                .name("Generic")
                .ns("Ns")
                .args(vec![T::get_output_ref()])
                .build()
        }

        fn visit_ser_exports(set: &mut HashSet<ZodExport>) {
            let export = ZodExport::builder()
                .name("Generic")
                .args(&["T"])
                .value(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("inner")
                            .value(Reference::builder().ns("Ns").name("T").build())
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
        fn get_input_ref() -> Reference {
            Reference::builder()
                .ns("Ns")
                .name("Generic")
                .args(vec![T::get_input_ref()])
                .build()
        }

        fn visit_de_exports(set: &mut HashSet<ZodExport>) {
            let export = ZodExport::builder()
                .name("Generic")
                .args(&["T"])
                .value(
                    ZodObject::builder()
                        .fields(vec![ZodNamedField::builder()
                            .name("inner")
                            .value(Reference::builder().ns("todo").name("T").build())
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
        fn get_output_ref() -> Reference {
            <String as OutputType>::get_output_ref()
        }
        fn visit_ser_exports(set: &mut HashSet<ZodExport>) {
            String::visit_ser_exports(set);
        }
    }

    impl InputType for Transparent {
        fn get_input_ref() -> Reference {
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
        fn get_output_ref() -> Reference {
            Reference {
                name: String::from("Nested"),
                ns: String::from("Ns"),
                args: vec![T::get_output_ref()],
            }
        }

        fn visit_ser_exports(set: &mut HashSet<ZodExport>) {
            let exp = ZodExport::builder()
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
        fn get_input_ref() -> Reference {
            Reference {
                ns: String::from("Ns"),
                name: String::from("Nested"),
                args: vec![T::get_input_ref()],
            }
        }
        fn visit_de_exports(set: &mut HashSet<ZodExport>) {
            let exp = ZodExport::builder()
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
        fn get_output_ref() -> Reference {
            Reference {
                ns: String::from("Ns"),
                name: String::from("SerOnly"),
                args: Vec::new(),
            }
        }
        fn visit_ser_exports(_set: &mut HashSet<ZodExport>) {}
    }

    #[test]
    fn inline_transparent_ok() {
        assert_eq!(Ts(&Transparent::get_output_ref()).to_string(), "String");
        assert_eq!(Ts(&Transparent::get_input_ref()).to_string(), "U8");
    }

    #[test]
    fn debug() {
        let u8_export = u8::collect_all_exports().into_iter().next().unwrap();
        let string_export = String::collect_all_exports().into_iter().next().unwrap();

        let generic_export = ZodExport::builder()
            .name("Generic")
            .args(&["T"])
            .value(
                ZodObject::builder()
                    .fields(vec![ZodNamedField::builder()
                        .name("inner")
                        .value(Reference::builder().ns("todo").name("T").build())
                        .build()])
                    .build(),
            )
            .build();

        assert_eq!(
            Ts(&Generic::<Transparent>::get_output_ref()).to_string(),
            "Generic<String>"
        );

        assert_eq!(
            Ts(&Generic::<crate::const_str!('M', 'Y', '_', 'T')>::get_output_ref()).to_string(),
            "Generic<MY_T>"
        );

        assert_eq!(
            Ts(&Generic::<Transparent>::get_input_ref()).to_string(),
            "Generic<U8>"
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
                }]
            }
        );
    }
}
