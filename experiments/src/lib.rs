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

pub trait ReprSer {
    fn repr_ser() -> Reference;
}

pub trait ReprDe {
    fn repr_de() -> Reference;
}

pub trait ExportVisitor {
    fn visit_exports(_set: &mut HashSet<ZodExport>);

    fn collect_exports() -> HashSet<ZodExport> {
        let mut set = HashSet::new();
        Self::visit_exports(&mut set);
        set
    }
}

impl<const C: char, T: const_str::Chain> ReprSer for const_str::ConstStr<C, T> {
    fn repr_ser() -> Reference {
        Reference {
            name: Self::value().to_string(),
            args: Vec::new(),
        }
    }
}

impl<const C: char, T: const_str::Chain> ReprDe for const_str::ConstStr<C, T> {
    fn repr_de() -> Reference {
        Reference {
            name: Self::value().to_string(),
            args: Vec::new(),
        }
    }
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

    macro_rules! impl_both {
    ($name: literal, $t: ty, [$($args: ident),*], $($export: tt)*) => {
        impl<$($args: ReprSer),*> ReprSer for $t {
            fn repr_ser() -> Reference {
                Reference {
                    name: String::from($name),
                    args: vec![$($args::repr_ser()),*],
                }
            }
        }

        impl<$($args: ReprDe),*> ReprDe for $t {
            fn repr_de() -> Reference {
                Reference {
                    name: String::from($name),
                    args: vec![$($args::repr_de()),*],
                }
            }
        }
        impl<$($args: ExportVisitor),*> ExportVisitor for $t {

            fn visit_exports(set: &mut HashSet<ZodExport>) {

                if let Some(export) = {
                    $($export)*
                } {
                    set.insert(export);
                }

                $($args::visit_exports(set));*

            }
        }
    };
}

    struct Generic<T> {
        inner: T,
    }

    impl_both!(
        "Generic",
        Generic<T>,
        [T],
        Some(
            ZodExport::builder()
                .name("Generic")
                .args(&["T"])
                .value(
                    ZodObject::builder()
                        .fields(vec![ZodObjectField::builder()
                            .name("inner")
                            .value(ZodTypeInner::Generic("T"))
                            .build()])
                        .build()
                )
                .build()
        )
    );

    struct Transparent;

    impl ExportVisitor for Transparent {
        fn visit_exports(set: &mut HashSet<ZodExport>) {
            String::visit_exports(set);
            u8::visit_exports(set);
        }
    }

    impl ReprSer for Transparent {
        fn repr_ser() -> Reference {
            <String as ReprSer>::repr_ser()
        }
    }

    impl ReprDe for Transparent {
        fn repr_de() -> Reference {
            <u8 as ReprDe>::repr_de()
        }
    }

    struct Nested<T> {
        inner: Generic<T>,
    }

    impl<T: ReprSer + ReprDe + ExportVisitor> ExportVisitor for Nested<T> {
        fn visit_exports(set: &mut HashSet<ZodExport>) {
            let exp = ZodExport::builder()
                .name("Nested")
                .args(&["T"])
                .value(
                    ZodObject::builder()
                        .fields(vec![ZodObjectField::builder()
                            .name("inner")
                            .value(Generic::<crate::const_str!('T')>::repr_ser())
                            .build()])
                        .build(),
                )
                .build();

            set.insert(exp.into());

            T::visit_exports(set)
        }
    }

    impl<T: ReprSer> ReprSer for Nested<T> {
        fn repr_ser() -> Reference {
            Reference {
                name: String::from("Nested"),
                args: vec![T::repr_ser()],
            }
        }
    }

    impl<T: ReprDe> ReprDe for Nested<T> {
        fn repr_de() -> Reference {
            Reference {
                name: String::from("Nested"),
                args: vec![T::repr_de()],
            }
        }
    }

    struct SerOnly;

    impl ExportVisitor for SerOnly {
        fn visit_exports(_set: &mut HashSet<ZodExport>) {}
    }

    impl ReprSer for SerOnly {
        fn repr_ser() -> Reference {
            Reference {
                name: String::from("SerOnly"),
                args: Vec::new(),
            }
        }
    }

    #[test]
    fn inline_transparent_ok() {
        assert_eq!(Ts(&Transparent::repr_ser()).to_string(), "String");
        assert_eq!(Ts(&Transparent::repr_de()).to_string(), "U8");
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
                    .fields(vec![ZodObjectField::builder()
                        .name("inner")
                        .value(ZodTypeInner::Generic("T"))
                        .build()])
                    .build(),
            )
            .build();

        assert_eq!(
            Ts(&Generic::<Transparent>::repr_ser()).to_string(),
            "Generic<String>"
        );

        assert_eq!(
            Ts(&Generic::<crate::const_str!('M', 'Y', '_', 'T')>::repr_ser()).to_string(),
            "Generic<MY_T>"
        );

        assert_eq!(
            Ts(&Generic::<Transparent>::repr_de()).to_string(),
            "Generic<U8>"
        );

        assert_eq!(
            <Generic::<u8>>::collect_exports(),
            [u8_export.clone(), generic_export.clone()]
                .into_iter()
                .collect()
        );

        assert_eq!(
            Transparent::collect_exports(),
            [u8_export.clone(), string_export.clone(),]
                .into_iter()
                .collect()
        );

        assert_eq!(
            <Generic::<Transparent>>::collect_exports(),
            [
                u8_export.clone(),
                string_export.clone(),
                generic_export.clone()
            ]
            .into_iter()
            .collect()
        );

        assert_eq!(
            <Generic::<SerOnly>>::collect_exports(),
            [generic_export.clone()].into_iter().collect()
        );

        assert_eq!(
            <Generic::<SerOnly>>::repr_ser(),
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
