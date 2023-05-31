use std::{fmt::Display, marker::PhantomData};

pub trait Chain {
    fn format(_: &mut std::fmt::Formatter<'_>) {}
}

pub struct ConstStr<const C: char, T> {
    _t: PhantomData<T>,
}

impl<const C: char, T> ConstStr<C, T> {
    pub fn value() -> Formatter<Self> {
        Formatter::default()
    }
}

impl<const C: char, T: Chain> Chain for ConstStr<C, T> {
    fn format(f: &mut std::fmt::Formatter<'_>) {
        use std::fmt::Write;
        f.write_char(C).unwrap();
        T::format(f)
    }
}

pub struct End;

impl Chain for End {}

pub struct Formatter<T> {
    _inner: PhantomData<T>,
}

impl<T> Default for Formatter<T> {
    fn default() -> Self {
        Self {
            _inner: PhantomData,
        }
    }
}

impl<T> From<Formatter<T>> for String
where
    T: Chain,
{
    fn from(value: Formatter<T>) -> Self {
        format!("{}", value)
    }
}

impl<T> Display for Formatter<T>
where
    T: Chain,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        T::format(f);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        type Abc = ConstStr<'a', ConstStr<'b', ConstStr<'c', End>>>;
        type Xyz = ConstStr<'x', ConstStr<'y', ConstStr<'z', End>>>;

        assert_eq!(format!("{}", Abc::value()), "abc");
        assert_eq!(format!("{}", Xyz::value()), "xyz");

        type X = crate::test_utils::const_str!('a', 'b', 'c');

        assert_eq!(format!("{}", X::value()), "abc");
    }
}
