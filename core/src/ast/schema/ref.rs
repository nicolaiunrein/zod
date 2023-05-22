use crate::ast::{Compiler, Delimited, Path};
use crate::{RequestType, ResponseType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ref {
    Resolved { path: Path, args: &'static [Ref] },
    Generic(usize),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResolvedRef {
    Resolved { path: Path, args: Vec<ResolvedRef> },
    Generic { name: &'static str },
}

impl Ref {
    // pub const fn generic(name: &'static str) -> Self {
    //     Self::Generic { name }
    // }

    pub const fn new_req<T: RequestType>() -> Self {
        let path = T::EXPORT.path;
        let args = T::ARGS;

        Self::Resolved { path, args }
    }

    pub const fn new_res<T: ResponseType>() -> Self {
        let path = T::EXPORT.path;
        let args = T::ARGS;

        Self::Resolved { path, args }
    }

    pub const fn new_stream_res<F, S, I>(_: &'static F) -> Self
    where
        F: Fn() -> S,
        S: futures::Stream<Item = I>,
        I: ResponseType,
    {
        Self::new_res::<I>()
    }

    pub const fn args(&self) -> &'static [Ref] {
        match self {
            Ref::Resolved { args, .. } => args,
            Ref::Generic { .. } => &[],
        }
    }

    pub(crate) fn resolve(&self, generics: &[&'static str]) -> ResolvedRef {
        let res = match self {
            Ref::Resolved { path, args } => ResolvedRef::Resolved {
                path: *path,
                args: args.iter().map(|r| r.resolve(generics)).collect::<Vec<_>>(),
            },
            Ref::Generic(index) => ResolvedRef::Generic {
                name: generics[*index], //todo
            },
        };
        res
    }
}

impl ResolvedRef {
    pub const fn get_generic(&self) -> Option<&'static str> {
        match self {
            ResolvedRef::Resolved { path, args } => None,
            ResolvedRef::Generic { name } => Some(name),
        }
    }
}

impl Compiler for ResolvedRef {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedRef::Resolved { path, args, .. } => {
                std::fmt::Display::fmt(&path, f)?;
                if !args.is_empty() {
                    f.write_str("(")?;
                    args.iter().comma_separated(f, |f, arg| arg.fmt_zod(f))?;
                    f.write_str(")")?;
                }
            }
            ResolvedRef::Generic { name } => f.write_str(name)?,
        }

        Ok(())
    }

    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedRef::Resolved { path, args, .. } => {
                std::fmt::Display::fmt(&path, f)?;
                if !args.is_empty() {
                    f.write_str("<")?;
                    args.iter().comma_separated(f, |f, arg| arg.fmt_ts(f))?;
                    f.write_str(">")?;
                }
            }
            ResolvedRef::Generic { name } => f.write_str(name)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_stream_res_ok() {
        struct MyStruct;

        impl MyStruct {
            fn test(_a: String) -> impl futures::Stream<Item = u8> {
                futures::stream::once(async { 0 })
            }
        }

        #[allow(unreachable_code)]
        let x = Ref::new_stream_res(&|| MyStruct::test(unreachable!()));

        assert_eq!(x, Ref::new_res::<u8>())
    }
}
