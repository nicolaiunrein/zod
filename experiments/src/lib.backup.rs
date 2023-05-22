#![allow(dead_code)]

pub mod context {
    use super::*;

    pub struct Both(pub InlineTypeDef);
    pub struct Ser {
        pub ser: InlineTypeDef,
    }
    pub struct De {
        pub de: InlineTypeDef,
    }
}

trait Export: Inline<context::Both> {
    fn export() -> String;
}

trait Inline<T> {
    fn inline() -> T;
}

macro_rules! impl_both {
    ($name: literal, $t: ty,  [$($args: ident),*]) => {
        impl<$($args: Inline<context::Both>),*> Inline<context::Both> for $t {
            fn inline() -> context::Both {
                context::Both(InlineTypeDef::Resolved {
                    name: $name,
                    args: vec![$($args::inline().0),*],
                })
            }
        }

        impl_ser!($name, $t, [$($args),*]);
        impl_de!($name, $t, [$($args),*]);
    };
}

macro_rules! impl_ser {
    ($name: literal, $t: ty,  [$($args: ident),*]) => {
        impl<$($args: Inline<context::Ser>),*> Inline<context::Ser> for $t {
            fn inline() -> context::Ser {
                context::Ser {
                    ser: InlineTypeDef::Resolved {
                      name: $name,
                      args: vec![$($args::inline().ser),*],
                    }
                }
            }
        }
    };
}

macro_rules! impl_de {
    ($name: literal, $t: ty,  [$($args: ident),*]) => {
        impl<$($args: Inline<context::De>),*> Inline<context::De> for $t {
            fn inline() -> context::De {
                context::De {
                    de: InlineTypeDef::Resolved {
                      name: $name,
                      args: vec![$($args::inline().de),*],
                    }
                }
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
pub enum InlineTypeDef {
    Resolved {
        name: &'static str,
        args: Vec<InlineTypeDef>,
    },
    Generic(usize),
}

impl InlineTypeDef {
    fn format(&self, generics: &[&'static str]) -> String {
        match self {
            InlineTypeDef::Resolved { name, args } => {
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

            InlineTypeDef::Generic(index) => generics[*index].to_string(),
        }
    }
}

/// This type implements Inline but not Export. It is used ...
struct Placeholder<const N: usize>;

impl<const N: usize> Inline<context::Both> for Placeholder<N> {
    fn inline() -> context::Both {
        context::Both(InlineTypeDef::Generic(N))
    }
}

// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
//
// test impl ------------------------------------------------------------

impl_both!("String", String, []);

impl Export for String {
    fn export() -> String {
        String::from("export type String = string;")
    }
}

// test impl ------------------------------------------------------------
//
impl_both!("u8", u8, []);

impl Export for u8 {
    fn export() -> String {
        String::from("export type u8 = number;")
    }
}

// test impl ------------------------------------------------------------
struct Transparent;

impl Inline<context::Ser> for Transparent {
    fn inline() -> context::Ser {
        context::Ser {
            ser: <String as Inline<context::Ser>>::inline().ser,
        }
    }
}

impl Inline<context::De> for Transparent {
    fn inline() -> context::De {
        context::De {
            de: <u8 as Inline<context::De>>::inline().de,
        }
    }
}

struct Generic<T> {
    inner: T,
}

impl_both!("Generic", Generic<T>, [T]);

impl<T: Inline<context::Both>> Export for Generic<T> {
    fn export() -> String {
        format!(
            "export interface Generic<T> {{ inner: {} }}",
            T::inline().0.format(&["T"])
        )
    }
}

struct Nested<T> {
    inner: Generic<T>,
}

impl<T: Inline<context::Both>> Export for Nested<T> {
    fn export() -> String {
        format!(
            "export interface Nested<T> {{ inner: {} }}",
            Generic::<Placeholder<0>>::inline().0.format(&["T"])
        )
    }
}

impl<T: Inline<context::Both>> Inline<context::Both> for Nested<T> {
    fn inline() -> context::Both {
        context::Both(InlineTypeDef::Resolved {
            name: "Nested",
            args: vec![T::inline().0],
        })
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
        assert_eq!(
            <Transparent as Inline<context::Ser>>::inline()
                .ser
                .format(&[]),
            "String"
        );
        assert_eq!(
            <Transparent as Inline<context::De>>::inline()
                .de
                .format(&[]),
            "u8"
        );
    }

    #[test]
    fn debug() {
        assert_eq!(
            <Generic::<Transparent> as Inline<context::Ser>>::inline()
                .ser
                .format(&[]),
            "Generic<String>"
        );

        assert_eq!(
            <Generic::<Transparent> as Inline<context::De>>::inline()
                .de
                .format(&[]),
            "Generic<u8>"
        );
    }
}
