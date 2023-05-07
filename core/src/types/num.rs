//! NewType numbers above 32-bits
//!
//! This module adds wrapper types for integers which are too big to be numbers in
//! javascript/typescript and instead exposes them as BigInts

use super::macros::impl_primitive;

macro_rules! impl_conversion {
    ($ty: ty, $name: ident) => {
        #[derive(
            serde::Deserialize,
            serde::Serialize,
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Default,
            Ord,
        )]
        #[serde(transparent)]
        pub struct $name(#[serde(with = "string")] pub $ty);

        impl std::convert::From<$name> for $ty {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl std::convert::From<$ty> for $name {
            fn from(value: $ty) -> Self {
                Self(value)
            }
        }

        impl std::ops::Deref for $name {
            type Target = $ty;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl PartialEq<$ty> for $name {
            fn eq(&self, other: &$ty) -> bool {
                &self.0 == other
            }
        }

        impl PartialOrd<$ty> for $name {
            fn partial_cmp(&self, other: &$ty) -> Option<std::cmp::Ordering> {
                Some(self.0.cmp(other))
            }
        }

        // TODO impl more traits
    };
}

impl_conversion!(u64, U64);
impl_conversion!(u128, U128);
impl_conversion!(usize, Usize);

impl_conversion!(i64, I64);
impl_conversion!(i128, I128);
impl_conversion!(isize, Isize);

mod string {
    use std::fmt::Display;
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

impl_primitive!({
    ty: crate::types::U64,
    name: "U64",
    ts: "number",
    zod: "z.coerce.bigint().nonnegative().lt(2n ** 64n)"
});

impl_primitive!({
    ty: crate::types::U128,
    name: "U128",
    ts: "number",
    zod: "z.coerce.bigint().nonnegative().lt(2n ** 128n)"
});

#[cfg(target_pointer_width = "64")]
impl_primitive!({
    ty: crate::types::Usize,
    name: "Usize",
    ts: "BigInt",
    zod: "z.coerce.bigint().nonnegative().lt(2n ** 64n)"
});

#[cfg(target_pointer_width = "32")]
impl_primitive!({
    ty: crate::types::Usize,
    name: "Usize",
    ts: "BigInt",
    zod: "z.coerce.bigint().nonnegative().lt(2n ** 32n)"
});

#[cfg(target_pointer_width = "16")]
impl_primitive!({
    ty: crate::types::Usize,
    name: "Usize",
    ts: "BigInt",
    zod: "z.coerce.bigint().nonnegative().lt(2n ** 16n)"
});

impl_primitive!({
    ty: crate::types::I64,
    name: "I64",
    ts: "number",
    zod: "z.coerce.bigint().gte(-(2n ** 63n)).lt(2n ** 63n)"
});

impl_primitive!({
    ty: crate::types::I128,
    name: "I128",
    ts: "number",
    zod: "z.coerce.bigint().gte(-(2n ** 127n)).lt(2n ** 127n)"
});

#[cfg(target_pointer_width = "64")]
impl_primitive!({
    ty: crate::types::Isize,
    name: "Isize",
    ts: "number",
    zod: "z.coerce.bigint().gte(-(2n ** 63n)).lt(2n ** 63n)"
});

#[cfg(target_pointer_width = "32")]
impl_primitive!({
    ty: crate::types::Isize,
    name: "Isize",
    ts: "number",
    zod: "z.coerce.bigint().gte(-(2n ** 31n)).lt(2n ** 31n)"
});

#[cfg(target_pointer_width = "16")]
impl_primitive!({
    ty: crate::types::Isize,
    name: "Isize",
    ts: "number",
    zod: "z.coerce.bigint().gte(-(2n ** 15n)).lt(2n ** 15n)"
});

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ok() {
        let res: Isize = serde_json::from_str("\"-123123123123\"").unwrap();
        assert_eq!(res, -123123123123);
    }
}
