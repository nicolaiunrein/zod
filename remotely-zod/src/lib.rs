pub use remotely_zod_derive::*;

pub trait Codegen {
    fn code() -> String;

    fn name() -> Option<&'static str> {
        None
    }

    fn compose() -> String {
        Self::name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| Self::code())
    }
}

macro_rules! impl_primitive {
    ($T:ty, $code: literal) => {
        impl Codegen for $T {
            fn code() -> String {
                String::from($code)
            }
        }
    };
}

macro_rules! impl_shadow {
    ($s:ty; $($impl:tt)*) => {
        $($impl)* {
            fn code() -> String {
                String::from(<$s>::code())
            }
        }
    };
}

macro_rules! impl_tuples {
    ( impl $($i:ident),* ) => {
        impl<$($i: Codegen),*> Codegen for ($($i,)*) {
            fn code() -> String {
                format!("z.tuple([{}])", vec![$($i::code()),*].join(", "))
            }
        }
    };
    ( $i2:ident $(, $i:ident)* ) => {
        impl_tuples!(impl $i2 $(, $i)* );
        impl_tuples!($($i),*);
    };
    () => {};
}

macro_rules! impl_wrapper {
    ($($t:tt)*) => {
        $($t)* {
            fn code() -> String {
                T::code()
            }
        }
    };
}

impl_primitive!(String, "z.string()");
impl_primitive!(&str, "z.string()");

impl_primitive!(u8, "z.number().finite().int().nonnegative().lte(255)");
impl_primitive!(u16, "z.number().finite().int().nonnegative().lte(65535)");
impl_primitive!(
    u32,
    "z.number().finite().int().nonnegative().lte(4294967295)"
);
impl_primitive!(
    u64,
    "z.number().finite().int().nonnegative().lte(18446744073709551615)"
);
impl_primitive!(
    u128,
    "z.number().finite().int().nonnegative().lte(340282366920938463463374607431768211455)"
);
impl_primitive!(usize, "z.number().finite().int().nonnegative()");

impl_primitive!(i8, "z.number().finite().int().lte(127).gte(-128)");
impl_primitive!(i16, "z.number().finite().int().lte(32767).gte(-32768)");
impl_primitive!(
    i32,
    "z.number().finite().int().lte(2147483647).gte(-2147483648)"
);
impl_primitive!(
    i64,
    "z.number().finite().int().lte(9223372036854775807).gte(-9223372036854775808)"
);
impl_primitive!(i128, "z.number().finite().int().lte(170141183460469231731687303715884105727).gte(-170141183460469231731687303715884105728)");
impl_primitive!(isize, "z.number().finitie().int()");

impl_primitive!(f32, "z.number()");
impl_primitive!(f64, "z.number()");

impl_primitive!(bool, "z.bool()");
impl_primitive!(char, "z.string().length(1)");
impl_primitive!((), "z.undefined()");

impl_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30
);

impl_wrapper!(impl<T: Codegen> Codegen for Box<T>);
impl_wrapper!(impl<T: Codegen> Codegen for std::sync::Arc<T>);
impl_wrapper!(impl<T: Codegen> Codegen for std::rc::Rc<T>);
impl_wrapper!(impl<T: Codegen + ToOwned> Codegen for std::borrow::Cow<'static, T>);
impl_wrapper!(impl<T: Codegen> Codegen for std::cell::Cell<T>);
impl_wrapper!(impl<T: Codegen> Codegen for std::cell::RefCell<T>);
impl_wrapper!(impl<T: Codegen> Codegen for std::sync::Mutex<T>);
impl_wrapper!(impl<T: Codegen> Codegen for std::sync::Weak<T>);
impl_wrapper!(impl<T: Codegen> Codegen for std::marker::PhantomData<T>);

impl_shadow!(Vec<T>; impl<T: Codegen> Codegen for std::collections::HashSet<T>);
impl_shadow!(Vec<T>; impl<T: Codegen> Codegen for std::collections::BTreeSet<T>);
impl_shadow!(std::collections::HashMap<K, V>; impl<K: Codegen, V: Codegen> Codegen for std::collections::BTreeMap<K, V>);
impl_shadow!(Vec<T>; impl<T: Codegen, const N: usize> Codegen for [T; N]);

impl<T: Codegen> Codegen for Vec<T> {
    fn code() -> String {
        format!("z.array({})", T::code())
    }
}

impl<K: Codegen, V: Codegen> Codegen for std::collections::HashMap<K, V> {
    fn code() -> String {
        format!("z.map({}, {})", K::code(), V::code())
    }
}

// #[cfg(test)]
// mod test {
// use super::*;

// #[test]
// fn it_works() {
// type X = std::collections::HashMap<
// (u32, String),
// Vec<(
// i8,
// std::collections::HashSet<std::marker::PhantomData<std::sync::Mutex<String>>>,
// )>,
// >;

// let code = X::code();
// assert_eq!(code, "");
// }
// }
