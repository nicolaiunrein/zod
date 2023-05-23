#![allow(dead_code)]
mod const_str;
mod utils;

use std::{collections::HashSet, fmt::Display};

trait Type {
    fn ref_ser() -> Inlined;
    fn ref_de() -> Inlined;

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

impl<const C: char, T: const_str::Tail> Type for const_str::ConstStr<C, T> {
    fn ref_ser() -> Inlined {
        Inlined {
            name: Self::value().to_string(),
            args: Vec::new(),
        }
    }

    fn ref_de() -> Inlined {
        Inlined {
            name: Self::value().to_string(),
            args: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Inlined {
    name: String,
    args: Vec<Inlined>,
}

impl Display for Inlined {
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
        impl<$($args: Type),*> Type for $t {
            fn ref_ser() -> Inlined {
                Inlined {
                    name: String::from($name),
                    args: vec![$($args::ref_ser()),*],
                }
            }

            fn ref_de() -> Inlined {
                Inlined {
                    name: String::from($name),
                    args: vec![$($args::ref_de()),*],
                }
            }

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
        fn ref_ser() -> Inlined {
            <String as Type>::ref_ser()
        }

        fn ref_de() -> Inlined {
            <u8 as Type>::ref_ser()
        }

        fn visit_exports_ser(set: &mut HashSet<String>) {
            String::visit_exports_ser(set);
        }

        fn visit_exports_de(set: &mut HashSet<String>) {
            u8::visit_exports_de(set);
        }
    }

    struct Nested<T> {
        inner: Generic<T>,
    }

    impl<T: Type> Type for Nested<T> {
        fn visit_exports_ser(set: &mut HashSet<String>) {
            if T::ref_ser() == T::ref_de() {
                set.insert(format!(
                    "export interface Nested<T> {{ inner: {} }}",
                    Generic::<crate::const_str!('T')>::ref_ser()
                ));
            } else {
                T::visit_exports_ser(set)
            }
        }

        fn visit_exports_de(set: &mut HashSet<String>) {
            if T::ref_ser() == T::ref_de() {
                set.insert(format!(
                    "export interface Nested<T> {{ inner: {} }}",
                    Generic::<crate::const_str!('T')>::ref_ser()
                ));
            } else {
                T::visit_exports_de(set)
            }
        }

        fn ref_ser() -> Inlined {
            Inlined {
                name: String::from("Nested"),
                args: vec![T::ref_ser()],
            }
        }

        fn ref_de() -> Inlined {
            Inlined {
                name: String::from("Nested"),
                args: vec![T::ref_de()],
            }
        }
    }

    #[test]
    fn inline_transparent_ok() {
        assert_eq!(Transparent::ref_ser().to_string(), "String");
        assert_eq!(Transparent::ref_de().to_string(), "u8");
    }

    #[test]
    fn debug() {
        assert_eq!(
            Generic::<Transparent>::ref_ser().to_string(),
            "Generic<String>"
        );

        assert_eq!(
            Generic::<crate::const_str!('M', 'Y', '_', 'T')>::ref_ser().to_string(),
            "Generic<MY_T>"
        );

        assert_eq!(Generic::<Transparent>::ref_de().to_string(), "Generic<u8>");

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
    }
}
