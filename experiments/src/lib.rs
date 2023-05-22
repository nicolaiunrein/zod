#![allow(dead_code)]

use std::collections::HashSet;

trait Type {
    fn visit_exports_ser(set: &mut HashSet<String>);
    fn visit_exports_de(set: &mut HashSet<String>);
    const INLINE_SER: Ast;
    const INLINE_DE: Ast;

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

macro_rules! impl_both {
    ($name: literal, $t: ty, [$($args: ident),*], $($export: tt)*) => {
        impl<$($args: Type),*> Type for $t {
            const INLINE_SER: Ast = Ast::Concrete {
                    name: $name,
                    args: &[$($args::INLINE_SER),*],
            };

            const INLINE_DE: Ast = Ast::Concrete {
                name: $name,
                args: &[$($args::INLINE_DE),*],
            };

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Context {
    Serialize,
    Deserialize,
}

#[derive(Debug, PartialEq)]
pub enum Ast {
    Concrete {
        name: &'static str,
        args: &'static [Ast],
    },
    Generic(usize),
}

impl Ast {
    fn format(&self, generics: &[&'static str]) -> String {
        match self {
            Ast::Concrete { name, args } => {
                if args.is_empty() {
                    format!("{name}")
                } else {
                    format!(
                        "{name}<{}>",
                        args.iter()
                            .map(|ty| ty.format(generics))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }

            Ast::Generic(index) => generics[*index].to_string(), // todo
        }
    }
}

/// This type implements Inline but not Export. It is used ...
struct Placeholder<const N: usize>;

impl<const N: usize> Type for Placeholder<N> {
    fn visit_exports_ser(_: &mut HashSet<String>) {
        // do nothing
    }

    fn visit_exports_de(_: &mut HashSet<String>) {
        // do nothing
    }

    const INLINE_SER: Ast = Ast::Generic(N);
    const INLINE_DE: Ast = Ast::Generic(N);
}

// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
//

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
    const INLINE_SER: Ast = <String as Type>::INLINE_SER;
    const INLINE_DE: Ast = <u8 as Type>::INLINE_DE;

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
    const INLINE_SER: Ast = Ast::Concrete {
        name: "Nested",
        args: &[T::INLINE_SER],
    };

    const INLINE_DE: Ast = Ast::Concrete {
        name: "Nested",
        args: &[T::INLINE_DE],
    };

    fn visit_exports_ser(set: &mut HashSet<String>) {
        if T::INLINE_SER == T::INLINE_DE {
            set.insert(format!(
                "export interface Nested<T> {{ inner: {} }}",
                Generic::<Placeholder<0>>::INLINE_SER.format(&["T"])
            ));
        } else {
            T::visit_exports_ser(set)
        }
    }

    fn visit_exports_de(set: &mut HashSet<String>) {
        if T::INLINE_SER == T::INLINE_DE {
            set.insert(format!(
                "export interface Nested<T> {{ inner: {} }}",
                Generic::<Placeholder<0>>::INLINE_SER.format(&["T"])
            ));
        } else {
            T::visit_exports_de(set)
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

    #[test]
    fn inline_transparent_ok() {
        assert_eq!(Transparent::INLINE_SER.format(&[]), "String");
        assert_eq!(Transparent::INLINE_DE.format(&[]), "u8");
    }

    #[test]
    fn debug() {
        assert_eq!(
            Generic::<Transparent>::INLINE_SER.format(&[]),
            "Generic<String>"
        );

        assert_eq!(
            Generic::<Placeholder<0>>::INLINE_SER.format(&["MY_T"]),
            "Generic<MY_T>"
        );

        assert_eq!(Generic::<Transparent>::INLINE_DE.format(&[]), "Generic<u8>");

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
