#![allow(dead_code)]

struct Generic<const N: usize>;

impl<const N: usize> RequestTypeInline for Generic<N> {
    fn inline() -> String {
        format!("T{}", N)
    }
}

trait RequestTypeExport {
    fn export() -> String;
}

trait RequestTypeInline {
    fn inline() -> String;
}

struct MyStruct {}

struct GenericStruct<T, U> {
    inner: T,
    inner2: U,
}

type GenericStructFlipped<T0, T1> = GenericStruct<T1, T0>;

struct User {
    inner: GenericStruct<MyStruct, MyStruct>,
}

struct GenericUser<T> {
    inner: GenericStruct<T, MyStruct>,
    flipped: GenericStructFlipped<T, MyStruct>,
}

// problem

impl<T: RequestTypeInline> RequestTypeInline for GenericUser<T> {
    fn inline() -> String {
        format!("GenericUser<{}>", T::inline())
    }
}

impl<T: RequestTypeInline> RequestTypeExport for GenericUser<T> {
    fn export() -> String {
        format!(
            "export interface GenericUser<T0> {{ inner: {}, flipped: {} }}",
            GenericStruct::<Generic<0>, MyStruct>::inline(),
            GenericStructFlipped::<Generic<0>, MyStruct>::inline()
        )
    }
}

// Export

impl RequestTypeExport for MyStruct {
    fn export() -> String {
        String::from("export interface MyStruct {}")
    }
}

impl<T: RequestTypeExport, U: RequestTypeExport> RequestTypeExport for GenericStruct<T, U> {
    fn export() -> String {
        String::from("export interface GenericStruct<T, U> { inner: T, inner2: U }")
    }
}

impl RequestTypeExport for User {
    fn export() -> String {
        format!(
            "export interface User {{ inner: {} }}",
            GenericStruct::<MyStruct, MyStruct>::inline()
        )
    }
}

/// Inline

impl RequestTypeInline for MyStruct {
    fn inline() -> String {
        String::from("MyStruct")
    }
}

impl<T: RequestTypeInline, U: RequestTypeInline> RequestTypeInline for GenericStruct<T, U> {
    fn inline() -> String {
        format!("GenericStruct<{}, {}>", T::inline(), U::inline())
    }
}

impl RequestTypeInline for User {
    fn inline() -> String {
        String::from("User")
    }
}

impl RequestTypeInline for u8 {
    fn inline() -> String {
        String::from("u8")
    }
}

impl RequestTypeExport for u8 {
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
        assert_eq!(GenericUser::<u8>::inline(), "GenericUser<u8>");
    }

    #[test]
    fn export_ok() {
        assert_eq!(
            GenericUser::<u8>::export(),
            "export interface GenericUser<T0> { inner: GenericStruct<T0, MyStruct>, flipped: GenericStruct<MyStruct, T0> }"
        );
    }
}
