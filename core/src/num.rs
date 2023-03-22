//! # Number types
//! This module adds wrapper types for integers which are too big to be numbers in
//! javascript/typescript

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ok() {
        let res: Isize = serde_json::from_str("\"-123123123123\"").unwrap();
        assert_eq!(res, -123123123123);
    }
}
