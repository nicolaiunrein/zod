//! **_NOTE:_**  This crate is not ready for production yet!
#![deny(unsafe_code)]

pub mod ast;

#[cfg(feature = "rpc")]
pub mod rpc;

mod build_ins;
use std::{
    any::TypeId,
    collections::{BTreeMap, HashSet},
    fmt::Display,
};

use ast::ZodExport;
pub use build_ins::*;

pub(crate) struct Delimited<I>(pub I, pub &'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Inlined {
    ConstUsize(usize),
    ConstU8(u8),
    ConstU16(u16),
    ConstU32(u32),
    ConstU64(u64),
    ConstU128(u128),
    ConstIk8(i8),
    ConstI16(i16),
    ConstI32(i32),
    ConstI64(i64),
    ConstI128(i128),
    ConstIsize(isize),
    ConstChar(char),
    ConstBool(bool),
    Type {
        ns: &'static str,
        name: &'static str,
        params: &'static [Inlined],
    },
}

impl Display for Inlined {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Inlined::Type {
                ns,
                name,
                ref params,
            } => {
                f.write_str(ns)?;
                f.write_str(".")?;
                f.write_str(name)?;

                if !params.is_empty() {
                    f.write_str("(")?;
                    Delimited(*params, ", ").fmt(f)?;
                    f.write_str(")")?;
                }

                Ok(())
            }

            Inlined::ConstUsize(inner) => f.write_fmt(format_args!("{}", inner)),
            Inlined::ConstU8(inner) => f.write_fmt(format_args!("{}", inner)),
            Inlined::ConstU16(inner) => f.write_fmt(format_args!("{}", inner)),
            Inlined::ConstU32(inner) => f.write_fmt(format_args!("{}", inner)),
            Inlined::ConstU64(inner) => f.write_fmt(format_args!("{}", inner)),
            Inlined::ConstU128(inner) => f.write_fmt(format_args!("{}", inner)),
            Inlined::ConstIk8(inner) => f.write_fmt(format_args!("{}", inner)),
            Inlined::ConstI16(inner) => f.write_fmt(format_args!("{}", inner)),
            Inlined::ConstI32(inner) => f.write_fmt(format_args!("{}", inner)),
            Inlined::ConstI64(inner) => f.write_fmt(format_args!("{}", inner)),
            Inlined::ConstI128(inner) => f.write_fmt(format_args!("{}", inner)),
            Inlined::ConstIsize(inner) => f.write_fmt(format_args!("{}", inner)),
            Inlined::ConstChar(inner) => f.write_fmt(format_args!("{}", inner)),
            Inlined::ConstBool(inner) => f.write_fmt(format_args!("{}", inner)),
        }
    }
}

pub trait ZodType: DependencyRegistration {
    const AST: ast::ZodExport;
    const INLINED: Inlined;
}

pub trait DependencyRegistration {
    fn register_dependencies(_: &mut DependencyMap)
    where
        Self: 'static;

    fn dependencies() -> DependencyMap
    where
        Self: 'static,
    {
        let mut cx = DependencyMap(Default::default());
        Self::register_dependencies(&mut cx);
        cx
    }
}

#[derive(Debug, PartialEq)]
pub struct DependencyMap(BTreeMap<TypeId, ZodExport>);

impl DependencyMap {
    pub fn add<T>(&mut self) -> bool
    where
        T: ZodType + 'static,
    {
        let id = TypeId::of::<T>();
        let node = T::AST;
        !self.0.insert(id, node).is_some()
    }

    pub fn resolve(self) -> HashSet<ZodExport> {
        self.0.into_iter().map(|(_, value)| value).collect()
    }
}

pub trait Namespace {
    const NAME: &'static str;
    const DOCS: Option<&'static str>;
    type Registry;

    fn generate() -> String
    where
        Self: 'static,
    {
        let mut out = String::from("export namespace ");
        out.push_str(Self::NAME);
        out.push_str(" { \n");

        //TODO ...

        out.push_str("}");
        out
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn nesting_ok() {
        assert_eq!(
            <Option<String>>::INLINED,
            Inlined::Type {
                ns: "Rs",
                name: "Option",
                params: &[String::INLINED]
            }
        );
        assert_eq!(
            <Result<std::collections::HashMap<usize, Option<bool>>, String>>::INLINED,
            Inlined::Type {
                ns: "Rs",
                name: "Result",
                params: &[
                    Inlined::Type {
                        ns: "Rs",
                        name: "HashMap",
                        params: &[
                            Inlined::Type {
                                ns: "Rs",
                                name: "Usize",
                                params: &[]
                            },
                            Inlined::Type {
                                ns: "Rs",
                                name: "Option",
                                params: &[Inlined::Type {
                                    ns: "Rs",
                                    name: "Bool",
                                    params: &[]
                                }]
                            }
                        ]
                    },
                    Inlined::Type {
                        ns: "Rs",
                        name: "String",
                        params: &[]
                    }
                ]
            }
        );

        assert_eq!(<[String; 5]>::INLINED.to_string(), "Rs.Array(5, Rs.String)");

        assert_eq!(
            <(String, usize, bool)>::INLINED,
            Inlined::Type {
                ns: "Rs",
                name: "Tuple3",
                params: &[
                    Inlined::Type {
                        ns: "Rs",
                        name: "String",
                        params: &[]
                    },
                    Inlined::Type {
                        ns: "Rs",
                        name: "Usize",
                        params: &[]
                    },
                    Inlined::Type {
                        ns: "Rs",
                        name: "Bool",
                        params: &[]
                    }
                ]
            }
        );
    }
}
