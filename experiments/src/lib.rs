#![allow(dead_code)]

trait Export: Inline {
    fn export() -> String;
}

trait InlineSer {
    fn inline_ser() -> InlineTypeDef;
}

trait InlineDe {
    fn inline_de() -> InlineTypeDef;
}

trait Inline: InlineSer + InlineDe {
    fn inline() -> InlineTypeDef {
        let ser = Self::inline_ser();
        let de = Self::inline_de();

        debug_assert_eq!(ser, de);
        ser
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Context {
    Serialize,
    Deserialize,
}

#[derive(Debug, PartialEq)]
enum InlineTypeDef {
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

impl<const N: usize> Inline for Placeholder<N> {
    fn inline() -> InlineTypeDef {
        InlineTypeDef::Generic(N)
    }
}

impl<const N: usize> InlineSer for Placeholder<N> {
    fn inline_ser() -> InlineTypeDef {
        InlineTypeDef::Generic(N)
    }
}

impl<const N: usize> InlineDe for Placeholder<N> {
    fn inline_de() -> InlineTypeDef {
        InlineTypeDef::Generic(N)
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

impl InlineSer for String {
    fn inline_ser() -> InlineTypeDef {
        InlineTypeDef::Resolved {
            name: "String",
            args: vec![],
        }
    }
}

impl InlineDe for String {
    fn inline_de() -> InlineTypeDef {
        InlineTypeDef::Resolved {
            name: "String",
            args: vec![],
        }
    }
}

impl Inline for String {}

impl Export for String {
    fn export() -> String {
        String::from("export type String = string;")
    }
}

// test impl ------------------------------------------------------------

impl InlineSer for u8 {
    fn inline_ser() -> InlineTypeDef {
        InlineTypeDef::Resolved {
            name: "u8",
            args: vec![],
        }
    }
}

impl InlineDe for u8 {
    fn inline_de() -> InlineTypeDef {
        InlineTypeDef::Resolved {
            name: "u8",
            args: vec![],
        }
    }
}

impl Inline for u8 {}

impl Export for u8 {
    fn export() -> String {
        String::from("export type u8 = number;")
    }
}

// test impl ------------------------------------------------------------
struct Transparent;

impl InlineSer for Transparent {
    fn inline_ser() -> InlineTypeDef {
        String::inline_ser()
    }
}

impl InlineDe for Transparent {
    fn inline_de() -> InlineTypeDef {
        u8::inline_ser()
    }
}

struct Generic<T> {
    inner: T,
}

impl<T: InlineSer> InlineSer for Generic<T> {
    fn inline_ser() -> InlineTypeDef {
        InlineTypeDef::Resolved {
            name: "Generic",
            args: vec![T::inline_ser()],
        }
    }
}

impl<T: InlineDe> InlineDe for Generic<T> {
    fn inline_de() -> InlineTypeDef {
        InlineTypeDef::Resolved {
            name: "Generic",
            args: vec![T::inline_de()],
        }
    }
}

impl<T: Inline> Inline for Generic<T> {}

impl<T: Inline> Export for Generic<T> {
    fn export() -> String {
        format!(
            "export interface Generic<T> {{ inner: {} }}",
            T::inline().format(&["T"])
        )
    }
}

struct Nested<T> {
    inner: Generic<T>,
}

impl<T: Inline> Export for Nested<T> {
    fn export() -> String {
        format!(
            "export interface Nested<T> {{ inner: {} }}",
            Generic::<Placeholder<0>>::inline().format(&["T"])
        )
    }
}

impl<T: InlineSer> InlineSer for Nested<T> {
    fn inline_ser() -> InlineTypeDef {
        InlineTypeDef::Resolved {
            name: "Nested",
            args: vec![T::inline_ser()],
        }
    }
}

impl<T: InlineDe> InlineDe for Nested<T> {
    fn inline_de() -> InlineTypeDef {
        InlineTypeDef::Resolved {
            name: "Nested",
            args: vec![T::inline_de()],
        }
    }
}

impl<T: Inline> Inline for Nested<T> {}

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
        assert_eq!(Transparent::inline_ser().format(&[]), "String");
        assert_eq!(Transparent::inline_de().format(&[]), "u8");
    }

    #[test]
    fn debug() {
        assert_eq!(
            Generic::<Transparent>::inline_ser().format(&[]),
            "Generic<String>"
        );

        assert_eq!(
            Generic::<Transparent>::inline_de().format(&[]),
            "Generic<u8>"
        );
    }
}
