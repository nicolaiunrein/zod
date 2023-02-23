pub use remotely_zod_derive::*;

pub trait Codegen {
    fn schema() -> String;
    fn type_def() -> String;
    fn type_name() -> String {
        Self::type_def()
    }
}

macro_rules! impl_primitive {
    ($name:literal, $T:ty, $schema: literal) => {
        impl Codegen for $T {
            fn type_def() -> String {
                String::from($name)
            }

            fn schema() -> String {
                String::from($schema)
            }
        }
    };
}

macro_rules! impl_shadow {
    ($s:ty; $($impl:tt)*) => {
        $($impl)* {
            fn type_def() -> String {
                <$s>::type_def()
            }
            fn schema() -> String {
                String::from(<$s>::schema())
            }

        }
    };
}

macro_rules! impl_tuples {
    ( impl $($i:ident),* ) => {
        impl<$($i: Codegen),*> Codegen for ($($i,)*) {
            fn type_def() -> String {
                format!("[{}]", vec![$($i::type_def()),*].join(", "))
            }


            fn schema() -> String {
                format!("z.tuple([{}])", vec![$($i::schema()),*].join(", "))
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
            fn type_def() -> String {
                T::type_def()
            }

            fn schema() -> String {
                T::schema()
            }

        }
    };
}

impl_primitive!("string", String, "z.string()");
impl_primitive!("string", &str, "z.string()");

impl_primitive!(
    "number",
    u8,
    "z.number().finite().int().nonnegative().lte(255)"
);
impl_primitive!(
    "number",
    u16,
    "z.number().finite().int().nonnegative().lte(65535)"
);
impl_primitive!(
    "number",
    u32,
    "z.number().finite().int().nonnegative().lte(4294967295)"
);
impl_primitive!(
    "number",
    u64,
    "z.number().finite().int().nonnegative().lte(18446744073709551615)"
);
impl_primitive!(
    "number",
    u128,
    "z.number().finite().int().nonnegative().lte(340282366920938463463374607431768211455)"
);
impl_primitive!("number", usize, "z.number().finite().int().nonnegative()");

impl_primitive!("number", i8, "z.number().finite().int().lte(127).gte(-128)");
impl_primitive!(
    "number",
    i16,
    "z.number().finite().int().lte(32767).gte(-32768)"
);
impl_primitive!(
    "number",
    i32,
    "z.number().finite().int().lte(2147483647).gte(-2147483648)"
);
impl_primitive!(
    "number",
    i64,
    "z.number().finite().int().lte(9223372036854775807).gte(-9223372036854775808)"
);
impl_primitive!("number", i128, "z.number().finite().int().lte(170141183460469231731687303715884105727).gte(-170141183460469231731687303715884105728)");
impl_primitive!("number", isize, "z.number().finitie().int()");

impl_primitive!("number", f32, "z.number()");
impl_primitive!("number", f64, "z.number()");

impl_primitive!("boolean", bool, "z.bool()");
impl_primitive!("string", char, "z.string().length(1)");
impl_primitive!("null", (), "z.null()");

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

impl_shadow!(std::collections::HashSet<T>; impl<T: Codegen> Codegen for std::collections::BTreeSet<T>);
impl_shadow!(std::collections::HashMap<K, V>; impl<K: Codegen, V: Codegen> Codegen for std::collections::BTreeMap<K, V>);
impl_shadow!(Vec<T>; impl<T: Codegen, const N: usize> Codegen for [T; N]);

impl<T: Codegen> Codegen for Vec<T> {
    fn schema() -> String {
        format!("z.array({})", T::schema())
    }

    fn type_def() -> String {
        format!("Array<{}>", T::type_def())
    }
}

impl<T: Codegen> Codegen for std::collections::HashSet<T> {
    fn schema() -> String {
        format!("z.set({})", T::schema())
    }

    fn type_def() -> String {
        format!("Array<{}>", T::type_def())
    }
}

impl<K: Codegen, V: Codegen> Codegen for std::collections::HashMap<K, V> {
    fn schema() -> String {
        format!("z.map({}, {})", K::schema(), V::schema())
    }
    fn type_def() -> String {
        format!("Map<{}, {}>", K::type_def(), V::type_def())
    }
}
