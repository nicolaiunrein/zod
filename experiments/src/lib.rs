#![allow(dead_code)]
mod const_str;
mod utils;

use std::{collections::HashSet, fmt::Display};

trait ReprSer {
    fn repr_ser() -> Arg;
}

trait ReprDe {
    fn repr_de() -> Arg;
}

trait Type {
    /// override this method to register a types export and dependencies
    fn visit_exports_ser(_set: &mut HashSet<String>) {}

    /// override this method to register a types export and dependencies
    fn visit_exports_de(_set: &mut HashSet<String>) {}

    fn exports_ser() -> HashSet<String> {
        let mut set = HashSet::new();
        Self::visit_exports_ser(&mut set);
        set
    }

    fn exports_de() -> HashSet<String> {
        let mut set = HashSet::new();
        Self::visit_exports_de(&mut set);
        set
    }
}

impl<const C: char, T: const_str::Chain> ReprSer for const_str::ConstStr<C, T> {
    fn repr_ser() -> Arg {
        Arg {
            name: Self::value().to_string(),
            args: Vec::new(),
        }
    }
}

impl<const C: char, T: const_str::Chain> ReprDe for const_str::ConstStr<C, T> {
    fn repr_de() -> Arg {
        Arg {
            name: Self::value().to_string(),
            args: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Arg {
    name: String,
    args: Vec<Arg>,
}

impl Display for Arg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.args.is_empty() {
            f.write_fmt(format_args!("{}", self.name))
        } else {
            self.name.fmt(f)?;
            f.write_fmt(format_args!("<{}>", utils::Separated(", ", &self.args)))?;
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
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! impl_both {
    ($name: literal, $t: ty, [$($args: ident),*], $($export: tt)*) => {
        impl<$($args: ReprSer),*> ReprSer for $t {
            fn repr_ser() -> Arg {
                Arg {
                    name: String::from($name),
                    args: vec![$($args::repr_ser()),*],
                }
            }
        }

        impl<$($args: ReprDe),*> ReprDe for $t {
            fn repr_de() -> Arg {
                Arg {
                    name: String::from($name),
                    args: vec![$($args::repr_de()),*],
                }
            }
        }
        impl<$($args: Type),*> Type for $t {

            fn visit_exports_ser(set: &mut HashSet<String>) {

                if let Some(export) = {
                    $($export)*
                } {
                    set.insert(export);
                }

                $($args::visit_exports_ser(set));*

            }

            fn visit_exports_de(set: &mut HashSet<String>) {

                if let Some(export) = {
                    $($export)*
                } {
                    set.insert(export);
                }

                $($args::visit_exports_de(set));*

            }

        }
    };
}

    impl_both!(
        "String",
        String,
        [],
        Some(String::from("export type String = string;"))
    );

    impl_both!("u8", u8, [], Some(String::from("export type u8 = number;")));

    struct Generic<T> {
        inner: T,
    }

    impl_both!(
        "Generic",
        Generic<T>,
        [T],
        Some(String::from("export interface Generic<T> { inner: T }",))
    );

    struct Transparent;

    impl Type for Transparent {
        fn visit_exports_ser(set: &mut HashSet<String>) {
            String::visit_exports_ser(set);
        }

        fn visit_exports_de(set: &mut HashSet<String>) {
            u8::visit_exports_de(set);
        }
    }

    impl ReprSer for Transparent {
        fn repr_ser() -> Arg {
            <String as ReprSer>::repr_ser()
        }
    }

    impl ReprDe for Transparent {
        fn repr_de() -> Arg {
            <u8 as ReprDe>::repr_de()
        }
    }

    struct Nested<T> {
        inner: Generic<T>,
    }

    impl<T: ReprSer + ReprDe + Type> Type for Nested<T> {
        fn visit_exports_ser(set: &mut HashSet<String>) {
            set.insert(format!(
                "export interface Nested<T> {{ inner: {} }}",
                Generic::<crate::const_str!('T')>::repr_ser()
            ));

            T::visit_exports_ser(set)
        }

        fn visit_exports_de(set: &mut HashSet<String>) {
            set.insert(format!(
                "export interface Nested<T> {{ inner: {} }}",
                Generic::<crate::const_str!('T')>::repr_ser()
            ));
            T::visit_exports_de(set)
        }
    }

    impl<T: ReprSer> ReprSer for Nested<T> {
        fn repr_ser() -> Arg {
            Arg {
                name: String::from("Nested"),
                args: vec![T::repr_ser()],
            }
        }
    }

    impl<T: ReprDe> ReprDe for Nested<T> {
        fn repr_de() -> Arg {
            Arg {
                name: String::from("Nested"),
                args: vec![T::repr_de()],
            }
        }
    }

    struct SerOnly;

    impl Type for SerOnly {}

    impl ReprSer for SerOnly {
        fn repr_ser() -> Arg {
            Arg {
                name: String::from("SerOnly"),
                args: Vec::new(),
            }
        }
    }

    #[test]
    fn inline_transparent_ok() {
        assert_eq!(Transparent::repr_ser().to_string(), "String");
        assert_eq!(Transparent::repr_de().to_string(), "u8");
    }

    #[test]
    fn debug() {
        assert_eq!(
            Generic::<Transparent>::repr_ser().to_string(),
            "Generic<String>"
        );

        assert_eq!(
            Generic::<crate::const_str!('M', 'Y', '_', 'T')>::repr_ser().to_string(),
            "Generic<MY_T>"
        );

        assert_eq!(Generic::<Transparent>::repr_de().to_string(), "Generic<u8>");

        assert_eq!(
            <Generic::<u8>>::exports_de(),
            [
                String::from("export type u8 = number;"),
                String::from("export interface Generic<T> { inner: T }"),
            ]
            .into_iter()
            .collect()
        );

        assert_eq!(
            Transparent::exports_de(),
            [String::from("export type u8 = number;"),]
                .into_iter()
                .collect()
        );

        assert_eq!(
            Transparent::exports_ser(),
            [String::from("export type String = string;"),]
                .into_iter()
                .collect()
        );

        assert_eq!(
            <Generic::<Transparent>>::exports_de(),
            [
                String::from("export type u8 = number;"),
                String::from("export interface Generic<T> { inner: T }"),
            ]
            .into_iter()
            .collect()
        );

        assert_eq!(
            <Generic::<Transparent>>::exports_ser(),
            [
                String::from("export type String = string;"),
                String::from("export interface Generic<T> { inner: T }"),
            ]
            .into_iter()
            .collect()
        );

        assert_eq!(
            <Generic::<SerOnly>>::exports_ser(),
            [String::from("export interface Generic<T> { inner: T }"),]
                .into_iter()
                .collect()
        );

        assert_eq!(
            <Generic::<SerOnly>>::repr_ser(),
            Arg {
                name: String::from("Generic"),
                args: vec![Arg {
                    name: String::from("SerOnly"),
                    args: vec![]
                }]
            }
        );
    }
}
