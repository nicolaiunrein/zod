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
pub struct Inlined {
    pub ns: &'static str,
    pub name: &'static str,
    pub params: &'static [Inlined],
}

impl Display for Inlined {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.ns)?;
        f.write_str(".")?;
        f.write_str(self.name)?;

        if !self.params.is_empty() {
            f.write_str("(")?;
            Delimited(self.params, ", ").fmt(f)?;
            f.write_str(")")?;
        }

        Ok(())
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
            Inlined {
                ns: "Rs",
                name: "Option",
                params: &[String::INLINED]
            }
        );
        assert_eq!(
            <Result<std::collections::HashMap<usize, Option<bool>>, String>>::INLINED,
            Inlined {
                ns: "Rs",
                name: "Result",
                params: &[
                    Inlined {
                        ns: "Rs",
                        name: "HashMap",
                        params: &[
                            Inlined {
                                ns: "Rs",
                                name: "Usize",
                                params: &[]
                            },
                            Inlined {
                                ns: "Rs",
                                name: "Option",
                                params: &[Inlined {
                                    ns: "Rs",
                                    name: "Bool",
                                    params: &[]
                                }]
                            }
                        ]
                    },
                    Inlined {
                        ns: "Rs",
                        name: "String",
                        params: &[]
                    }
                ]
            }
        );

        assert_eq!(
            <[String; 5]>::INLINED.to_string(),
            "Rs.Array(5, Rs.String)" // Inlined {
                                     //     ns: "Rs",
                                     //     name: "Array",
                                     //     params: &[Inlined {
                                     //         ns: "Rs",
                                     //         name: "String",
                                     //         params: &[]
                                     //     }]
                                     // }
        );

        assert_eq!(
            <(String, usize, bool)>::INLINED,
            Inlined {
                ns: "Rs",
                name: "Tuple3",
                params: &[
                    Inlined {
                        ns: "Rs",
                        name: "String",
                        params: &[]
                    },
                    Inlined {
                        ns: "Rs",
                        name: "Usize",
                        params: &[]
                    },
                    Inlined {
                        ns: "Rs",
                        name: "Bool",
                        params: &[]
                    }
                ]
            }
        );
    }
}
