use crate::TsTypeDef;
use crate::ZodType;

macro_rules! impl_primitive {
    ($name:literal, $T:ty, $schema: literal) => {
        impl ZodType for $T {
            /// ```ts
            /// // TS-type
            #[doc = $name]
            /// ```
            fn type_def() -> TsTypeDef {
                TsTypeDef::Type(String::from($name))
            }

            /// ```ts
            /// // zod schema
            #[doc = $schema]
            /// ```
            fn schema() -> String {
                String::from($schema)
            }
        }
    };
}

macro_rules! impl_shadow {
    ($other:literal, $other_link:literal, $s:ty; $($impl:tt)*) => {
        $($impl)* {
            #[doc = concat!("shadows impl for [", $other, "](#impl-ZodType-for-", $other_link, ")")]
            fn type_def() -> TsTypeDef {
                <$s>::type_def()
            }

            #[doc = concat!("shadows impl for [", $other, "](#impl-ZodType-for-", $other_link, ")")]
            fn schema() -> String {
                String::from(<$s>::schema())
            }

        }
    };
}

macro_rules! impl_tuples {
    ( impl $($i:ident),* ) => {
        impl<$($i: ZodType),*> ZodType for ($($i,)*) {
            /// ```ts
            /// // TS-type
            /// [T1, T2, ... ]
            /// ```
            fn type_def() -> TsTypeDef {
                TsTypeDef::Type(format!("[{}]", vec![$($i::type_def().to_string()),*].join(", ")))
            }


            /// ```ts
            /// // zod schema
            /// z.tuple([T1, T2, ... ])
            /// ```
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
            /// ```ts
            /// // TS-type
            /// T
            /// ```
            fn type_def() -> TsTypeDef {
                T::type_def()
            }

            /// ```ts
            /// // zod schema
            /// T
            /// ```
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
impl_primitive!("number", isize, "z.number().finite().int()");

impl_primitive!("number", f32, "z.number()");
impl_primitive!("number", f64, "z.number()");

impl_primitive!("boolean", bool, "z.bool()");
impl_primitive!("string", char, "z.string().length(1)");
impl_primitive!("null", (), "z.null()");

impl_tuples!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21,
    T22, T23, T24, T25, T26, T27, T28, T29, T30
);

impl_wrapper!(impl<T: ZodType> ZodType for Box<T>);
impl_wrapper!(impl<T: ZodType> ZodType for std::sync::Arc<T>);
impl_wrapper!(impl<T: ZodType> ZodType for std::rc::Rc<T>);
impl_wrapper!(impl<T: ZodType + ToOwned> ZodType for std::borrow::Cow<'static, T>);
impl_wrapper!(impl<T: ZodType> ZodType for std::cell::Cell<T>);
impl_wrapper!(impl<T: ZodType> ZodType for std::cell::RefCell<T>);
impl_wrapper!(impl<T: ZodType> ZodType for std::sync::Mutex<T>);
impl_wrapper!(impl<T: ZodType> ZodType for std::sync::Weak<T>);
impl_wrapper!(impl<T: ZodType> ZodType for std::marker::PhantomData<T>);

impl_shadow!("HashSet", "HashSet&lt;T,+RandomState&gt;", std::collections::HashSet<T>; impl<T: ZodType> ZodType for std::collections::BTreeSet<T>);
impl_shadow!("HashMap", "HashMap&lt;K,+V,+RandomState&gt;", std::collections::HashMap<K, V>; impl<K: ZodType, V: ZodType> ZodType for std::collections::BTreeMap<K, V>);

impl_shadow!("Vec", "Vec&lt;T&gt;", Vec<T>; impl<T: ZodType, const N: usize> ZodType for [T; N]);

impl<T: ZodType> ZodType for Vec<T> {
    /// ```ts
    /// // TS-type
    /// Array<T>
    /// ```
    fn type_def() -> TsTypeDef {
        TsTypeDef::Type(format!("Array<{}>", T::type_def()))
    }

    /// ```ts
    /// // zod schema
    /// z.array(T)
    /// ```
    fn schema() -> String {
        format!("z.array({})", T::schema())
    }
}

impl<T: ZodType> ZodType for std::collections::HashSet<T> {
    /// ```ts
    /// // TS-type
    /// Set<T>
    /// ```
    fn type_def() -> TsTypeDef {
        TsTypeDef::Type(format!("Set<{}>", T::type_def()))
    }

    /// ```ts
    /// // zod schema
    /// z.set(T)
    /// ```
    fn schema() -> String {
        format!("z.set({})", T::schema())
    }
}

impl<K: ZodType, V: ZodType> ZodType for std::collections::HashMap<K, V> {
    /// ```ts
    /// // TS-type
    /// Map<K, V>
    /// ```
    fn type_def() -> TsTypeDef {
        TsTypeDef::Type(format!("Map<{}, {}>", K::type_def(), V::type_def()))
    }

    /// ```ts
    /// // zod schema
    /// z.map(K, V)
    /// ```
    fn schema() -> String {
        format!("z.map({}, {})", K::schema(), V::schema())
    }
}

impl<T: ZodType> ZodType for Option<T> {
    /// ```ts
    /// // TS-type
    /// T | undefined
    /// ```
    fn type_def() -> TsTypeDef {
        // Todo
        TsTypeDef::Type(format!("({} | undefined)", T::type_def()))
    }

    /// ```ts
    /// // zod schema
    /// T.optional()
    /// ```
    fn schema() -> String {
        format!("{}.optional()", T::schema())
    }
}

impl<T: ZodType, E: ZodType> ZodType for Result<T, E> {
    /// ```ts
    /// // TS-type
    /// { Ok: T } | { Err: E }
    /// ```
    fn type_def() -> TsTypeDef {
        TsTypeDef::Type(format!(
            "{{ Ok: {} }} | {{ Err: {} }}",
            T::type_def(),
            E::type_def()
        ))
    }

    /// ```ts
    /// // zod schema
    /// z.union([
    ///   z.object({ Ok: T }),
    ///   z.object({ Err: E })
    /// ])
    /// ```
    fn schema() -> String {
        format!(
            "z.union([z.object({{ Ok: {} }}), z.object({{ Err: {} }})])",
            T::schema(),
            E::schema()
        )
    }
}

#[cfg(test)]
mod test {
    use super::ZodType;

    #[test]
    fn result_ok() {
        type Res = Result<usize, String>;
        let res_ok: Result<usize, String> = Ok(1);
        let res_err: Result<usize, String> = Err(String::from("abc"));
        let json_ok = serde_json::to_value(res_ok).unwrap();
        let json_err = serde_json::to_value(res_err).unwrap();
        assert_eq!(json_ok, serde_json::json!({"Ok": 1}));
        assert_eq!(json_err, serde_json::json!({"Err": "abc"}));

        assert_eq!(
            Res::schema(),
            format!(
                "z.union([z.object({{ Ok: {} }}), z.object({{ Err: {} }})])",
                usize::schema(),
                String::schema(),
            )
        )
    }
}
