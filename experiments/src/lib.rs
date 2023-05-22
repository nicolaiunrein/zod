#![allow(dead_code)]

trait Type {
    type SIMPLIFIED: Export + Inline;
}

impl<T> Type for T
where
    T: Export + Inline,
{
    type SIMPLIFIED = Self;
}

/// This trait should be implemented so that it always returns the same String regardless of the
/// generic inputs
trait Export {
    fn export() -> String;
}

/// This trait is dependent on the resolved type ie. `MyStruct<u8>`::inline()` should differ from
/// `MyStruct<String>::inline()`
trait Inline {
    fn inline() -> InlineTypeDef;
}

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
                            .map(|tt| tt.format(generics))
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
struct GenericPlaceholder<const N: usize>;

impl<const N: usize> Inline for GenericPlaceholder<N> {
    fn inline() -> InlineTypeDef {
        InlineTypeDef::Generic(N)
    }
}

// test impl

struct MyStruct {}

struct GenericStruct<T, U> {
    inner: T,
    inner2: U,
}

type GenericStructFlipped<T0, T1> = GenericStruct<T1, T0>;

struct User {
    inner: GenericStruct<MyStruct, MyStruct>,
}

struct GenericUser<MyT> {
    inner: GenericStruct<MyT, MyStruct>,
    flipped: GenericStructFlipped<MyT, MyStruct>,
}

impl<T: Inline> Inline for GenericUser<T> {
    fn inline() -> InlineTypeDef {
        InlineTypeDef::Resolved {
            name: "GenericUser",
            args: vec![T::inline()],
        }
    }
}

impl<T: Inline> Export for GenericUser<T> {
    fn export() -> String {
        format!(
            "export interface GenericUser<MyT> {{ inner: {}, flipped: {} }}",
            GenericStruct::<GenericPlaceholder<0>, MyStruct>::inline().format(&["MyT"]),
            GenericStructFlipped::<GenericPlaceholder<0>, MyStruct>::inline().format(&["MyT"])
        )
    }
}

// Export

impl Export for MyStruct {
    fn export() -> String {
        String::from("export interface MyStruct {}")
    }
}

impl<T: Export, U: Export> Export for GenericStruct<T, U> {
    fn export() -> String {
        String::from("export interface GenericStruct<T, U> { inner: T, inner2: U }")
    }
}

impl Export for User {
    fn export() -> String {
        format!(
            "export interface User {{ inner: {} }}",
            GenericStruct::<MyStruct, MyStruct>::inline().format(&[])
        )
    }
}

/// Inline

impl Inline for MyStruct {
    fn inline() -> InlineTypeDef {
        InlineTypeDef::Resolved {
            name: "MyStruct",
            args: vec![],
        }
    }
}

impl<T: Inline, U: Inline> Inline for GenericStruct<T, U> {
    fn inline() -> InlineTypeDef {
        InlineTypeDef::Resolved {
            name: "GenericStruct",
            args: vec![T::inline(), U::inline()],
        }
    }
}

impl Inline for User {
    fn inline() -> InlineTypeDef {
        InlineTypeDef::Resolved {
            name: "User",
            args: vec![],
        }
    }
}

impl Inline for u8 {
    fn inline() -> InlineTypeDef {
        InlineTypeDef::Resolved {
            name: "u8",
            args: vec![],
        }
    }
}

impl Export for u8 {
    fn export() -> String {
        String::from("export type u8 = number;")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn inline_ok() {
        assert_eq!(
            GenericUser::<GenericStruct<User, u8>>::inline().format(&[]),
            "GenericUser<GenericStruct<User, u8>>"
        );
    }

    #[test]
    fn export_ok() {
        assert_eq!(
            GenericUser::<GenericStruct<User, u8>>::export(),
            "export interface GenericUser<MyT> { inner: GenericStruct<MyT, MyStruct>, flipped: GenericStruct<MyStruct, MyT> }"
        );
    }
}
