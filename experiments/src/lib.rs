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

use crate::types::Crate;

pub trait RefSer {
    fn ref_ser() -> Reference;
    fn visit_ser_exports(_set: &mut HashSet<ZodExport>);

    fn collect_ser_exports() -> HashSet<ZodExport> {
        let mut set = HashSet::new();
        Self::visit_ser_exports(&mut set);
        set
    }
}

pub trait RefDe {
    fn ref_de() -> Reference;
    fn visit_de_exports(_set: &mut HashSet<ZodExport>);

    fn collect_de_exports() -> HashSet<ZodExport> {
        let mut set = HashSet::new();
        Self::visit_de_exports(&mut set);
        set
    }
}

pub trait Type {
    fn reference() -> Reference;
    fn visit_exports(_set: &mut HashSet<ZodExport>);

    fn collect_exports() -> HashSet<ZodExport> {
        let mut set = HashSet::new();
        Self::visit_exports(&mut set);
        set
    }
}

impl<T> RefSer for T
where
    T: Type,
{
    fn ref_ser() -> Reference {
        Self::reference()
    }

    fn visit_ser_exports(set: &mut HashSet<ZodExport>) {
        Self::visit_exports(set)
    }
}

impl<T> RefDe for T
where
    T: Type,
{
    fn ref_de() -> Reference {
        Self::reference()
    }

    fn visit_de_exports(set: &mut HashSet<ZodExport>) {
        Self::visit_exports(set)
    }
}

impl<const C: char, T: const_str::Chain> Type for const_str::ConstStr<C, T> {
    fn reference() -> Reference {
        Reference {
            name: Self::value().to_string(),
            args: Vec::new(),
        }
    }

    fn visit_exports(_set: &mut HashSet<ZodExport>) {}
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Export {
    pub ts: String,
    pub zod: String,
}

impl Display for Export {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.ts)?;
        f.write_str(&self.zod)?;
        Ok(())
    }
}

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct Reference {
    #[builder(setter(into))]
    pub name: String,

    #[builder(default)]
    pub args: Vec<Reference>,
}

impl ToTokens for Reference {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = &self.name;
        let args = &self.args;
        tokens.extend(quote!(#Crate::Reference {
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

    impl<T> RefSer for Generic<T>
    where
        T: RefSer,
    {
        fn ref_ser() -> Reference {
            Reference::builder()
                .name("Generic")
                .args(vec![T::ref_ser()])
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
                            .value(Reference::builder().name("T").build())
                            .build()])
                        .build(),
                )
                .build();

            set.insert(export);
            T::visit_ser_exports(set)
        }
    }

    impl<T> RefDe for Generic<T>
    where
        T: RefDe,
    {
        fn ref_de() -> Reference {
            Reference::builder()
                .name("Generic")
                .args(vec![T::ref_de()])
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
                            .value(Reference::builder().name("T").build())
                            .build()])
                        .build(),
                )
                .build();

            set.insert(export);
            T::visit_de_exports(set)
        }
    }

    struct Transparent;

    impl RefSer for Transparent {
        fn ref_ser() -> Reference {
            <String as RefSer>::ref_ser()
        }
        fn visit_ser_exports(set: &mut HashSet<ZodExport>) {
            String::visit_ser_exports(set);
        }
    }

    impl RefDe for Transparent {
        fn ref_de() -> Reference {
            <u8 as RefDe>::ref_de()
        }
        fn visit_de_exports(set: &mut HashSet<ZodExport>) {
            u8::visit_de_exports(set);
        }
    }

    struct Nested<T> {
        inner: Generic<T>,
    }

    impl<T: RefSer> RefSer for Nested<T> {
        fn ref_ser() -> Reference {
            Reference {
                name: String::from("Nested"),
                args: vec![T::ref_ser()],
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
                            .value(Generic::<crate::const_str!('T')>::ref_ser())
                            .build()])
                        .build(),
                )
                .build();

            set.insert(exp.into());

            T::visit_ser_exports(set)
        }
    }

    impl<T: RefDe> RefDe for Nested<T> {
        fn ref_de() -> Reference {
            Reference {
                name: String::from("Nested"),
                args: vec![T::ref_de()],
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
                            .value(Generic::<crate::const_str!('T')>::ref_de())
                            .build()])
                        .build(),
                )
                .build();

            set.insert(exp.into());

            T::visit_de_exports(set)
        }
    }

    struct SerOnly;

    impl RefSer for SerOnly {
        fn ref_ser() -> Reference {
            Reference {
                name: String::from("SerOnly"),
                args: Vec::new(),
            }
        }
        fn visit_ser_exports(_set: &mut HashSet<ZodExport>) {}
    }

    #[test]
    fn inline_transparent_ok() {
        assert_eq!(Ts(&Transparent::ref_ser()).to_string(), "String");
        assert_eq!(Ts(&Transparent::ref_de()).to_string(), "U8");
    }

    #[test]
    fn debug() {
        let u8_export = u8::collect_exports().into_iter().next().unwrap();
        let string_export = String::collect_exports().into_iter().next().unwrap();

        let generic_export = ZodExport::builder()
            .name("Generic")
            .args(&["T"])
            .value(
                ZodObject::builder()
                    .fields(vec![ZodNamedField::builder()
                        .name("inner")
                        .value(Reference::builder().name("T").build())
                        .build()])
                    .build(),
            )
            .build();

        assert_eq!(
            Ts(&Generic::<Transparent>::ref_ser()).to_string(),
            "Generic<String>"
        );

        assert_eq!(
            Ts(&Generic::<crate::const_str!('M', 'Y', '_', 'T')>::ref_ser()).to_string(),
            "Generic<MY_T>"
        );

        assert_eq!(
            Ts(&Generic::<Transparent>::ref_de()).to_string(),
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
            <Generic::<SerOnly>>::ref_ser(),
            Reference {
                name: String::from("Generic"),
                args: vec![Reference {
                    name: String::from("SerOnly"),
                    args: vec![]
                }]
            }
        );
    }
}
