mod const_str;
mod utils;

use std::{collections::HashSet, fmt::Display};

pub trait ReprSer {
    fn repr_ser() -> Arg;
}

pub trait ReprDe {
    fn repr_de() -> Arg;
}

pub trait ExportVisitor {
    fn visit_exports(_set: &mut HashSet<Export>);

    fn collect_exports() -> HashSet<Export> {
        let mut set = HashSet::new();
        Self::visit_exports(&mut set);
        set
    }
}

impl<const C: char, T: const_str::Chain> ReprSer for const_str::ConstStr<C, T> {
    fn repr_ser() -> Arg {
        Arg {
            name: Self::value().to_string(),
            args: Vec::new(),
        }
    }
}

impl<const C: char, T: const_str::Chain> ReprDe for const_str::ConstStr<C, T> {
    fn repr_de() -> Arg {
        Arg {
            name: Self::value().to_string(),
            args: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Export {
    pub ts: String,
    pub zod: String,
}

impl Display for Export {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.ts)?;
        f.write_str(&self.zod)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Arg {
    pub name: String,
    pub args: Vec<Arg>,
}

struct TsArg<'a>(&'a Arg);
struct ZodArg<'a>(&'a Arg);

impl Arg {
    fn as_ts(&self) -> TsArg {
        TsArg(self)
    }

    fn as_zod(&self) -> ZodArg {
        ZodArg(self)
    }
}

impl<'a> Display for TsArg<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.args.is_empty() {
            f.write_fmt(format_args!("{}", self.0.name))
        } else {
            self.0.name.fmt(f)?;
            let args = self
                .0
                .args
                .iter()
                .map(|arg| arg.as_ts())
                .collect::<Vec<_>>();

            f.write_fmt(format_args!("<{}>", utils::Separated(", ", &args)))?;
            Ok(())
        }
    }
}

impl<'a> Display for ZodArg<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.args.is_empty() {
            f.write_fmt(format_args!("{}", self.0.name))
        } else {
            self.0.name.fmt(f)?;
            let args = self
                .0
                .args
                .iter()
                .map(|arg| arg.as_zod())
                .collect::<Vec<_>>();

            f.write_fmt(format_args!("({})", utils::Separated(", ", &args)))?;
            Ok(())
        }
    }
}

// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
// ------------------------------------------------------------
//
#[cfg(test)]
mod test {
    #![allow(dead_code)]
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! impl_both {
    ($name: literal, $t: ty, [$($args: ident),*], $($export: tt)*) => {
        impl<$($args: ReprSer),*> ReprSer for $t {
            fn repr_ser() -> Arg {
                Arg {
                    name: String::from($name),
                    args: vec![$($args::repr_ser()),*],
                }
            }
        }

        impl<$($args: ReprDe),*> ReprDe for $t {
            fn repr_de() -> Arg {
                Arg {
                    name: String::from($name),
                    args: vec![$($args::repr_de()),*],
                }
            }
        }
        impl<$($args: Exportable),*> Exportable for $t {

            fn visit_exports(set: &mut HashSet<Export>) {

                if let Some(export) = {
                    $($export)*
                } {
                    set.insert(export);
                }

                $($args::visit_exports(set));*

            }
        }
    };
}

    impl_both!(
        "String",
        String,
        [],
        Some(Export {
            ts: String::from("export type String = string;"),
            zod: String::from("export const String = z.string();")
        })
    );

    impl_both!(
        "u8",
        u8,
        [],
        Some(Export {
            ts: String::from("export type u8 = number;"),
            zod: String::from("export const u8 = z.number();")
        })
    );

    struct Generic<T> {
        inner: T,
    }

    impl_both!(
        "Generic",
        Generic<T>,
        [T],
        Some(Export {
            ts: String::from("export interface Generic<T> { inner: T }"),
            zod: String::from("export const Generic = (T: z.ZodTypeAny) => z.object({ inner: T })",)
        })
    );

    struct Transparent;

    impl ExportVisitor for Transparent {
        fn visit_exports(set: &mut HashSet<Export>) {
            String::visit_exports(set);
            u8::visit_exports(set);
        }
    }

    impl ReprSer for Transparent {
        fn repr_ser() -> Arg {
            <String as ReprSer>::repr_ser()
        }
    }

    impl ReprDe for Transparent {
        fn repr_de() -> Arg {
            <u8 as ReprDe>::repr_de()
        }
    }

    struct Nested<T> {
        inner: Generic<T>,
    }

    impl<T: ReprSer + ReprDe + ExportVisitor> ExportVisitor for Nested<T> {
        fn visit_exports(set: &mut HashSet<Export>) {
            set.insert(Export {
                ts: format!(
                    "export interface Nested<T> {{ inner: {} }}",
                    Generic::<crate::const_str!('T')>::repr_ser().as_ts()
                ),
                zod: format!(
                    "export const Nested = (T: z.ZodTypeAny) => z.object({{ inner: {} }})",
                    Generic::<crate::const_str!('T')>::repr_ser().as_zod()
                ),
            });

            T::visit_exports(set)
        }
    }

    impl<T: ReprSer> ReprSer for Nested<T> {
        fn repr_ser() -> Arg {
            Arg {
                name: String::from("Nested"),
                args: vec![T::repr_ser()],
            }
        }
    }

    impl<T: ReprDe> ReprDe for Nested<T> {
        fn repr_de() -> Arg {
            Arg {
                name: String::from("Nested"),
                args: vec![T::repr_de()],
            }
        }
    }

    struct SerOnly;

    impl ExportVisitor for SerOnly {
        fn visit_exports(_set: &mut HashSet<Export>) {}
    }

    impl ReprSer for SerOnly {
        fn repr_ser() -> Arg {
            Arg {
                name: String::from("SerOnly"),
                args: Vec::new(),
            }
        }
    }

    #[test]
    fn inline_transparent_ok() {
        assert_eq!(Transparent::repr_ser().as_ts().to_string(), "String");
        assert_eq!(Transparent::repr_de().as_ts().to_string(), "u8");
    }

    #[test]
    fn debug() {
        assert_eq!(
            Generic::<Transparent>::repr_ser().as_ts().to_string(),
            "Generic<String>"
        );

        assert_eq!(
            Generic::<crate::const_str!('M', 'Y', '_', 'T')>::repr_ser()
                .as_ts()
                .to_string(),
            "Generic<MY_T>"
        );

        assert_eq!(
            Generic::<Transparent>::repr_de().as_ts().to_string(),
            "Generic<u8>"
        );

        assert_eq!(
            <Generic::<u8>>::collect_exports(),
            [
                Export {
                    ts: String::from("export type u8 = number;"),
                    zod: String::from("export const u8 = z.number();"),
                },
                Export {
                    ts: String::from("export interface Generic<T> { inner: T }"),
                    zod: String::from(
                        "export const Generic = (T: z.ZodTypeAny) => z.object({ inner: T })"
                    ),
                }
            ]
            .into_iter()
            .collect()
        );

        assert_eq!(
            Transparent::collect_exports(),
            [
                Export {
                    ts: String::from("export type u8 = number;"),
                    zod: String::from("export const u8 = z.number();"),
                },
                Export {
                    ts: String::from("export type String = string;"),
                    zod: String::from("export const String = z.string();"),
                },
            ]
            .into_iter()
            .collect()
        );

        assert_eq!(
            <Generic::<Transparent>>::collect_exports(),
            [
                Export {
                    ts: String::from("export type u8 = number;"),
                    zod: String::from("export const u8 = z.number();"),
                },
                Export {
                    ts: String::from("export type String = string;"),
                    zod: String::from("export const String = z.string();"),
                },
                Export {
                    ts: String::from("export interface Generic<T> { inner: T }"),
                    zod: String::from(
                        "export const Generic = (T: z.ZodTypeAny) => z.object({ inner: T })"
                    ),
                }
            ]
            .into_iter()
            .collect()
        );

        assert_eq!(
            <Generic::<SerOnly>>::collect_exports(),
            [Export {
                ts: String::from("export interface Generic<T> { inner: T }"),
                zod: String::from(
                    "export const Generic = (T: z.ZodTypeAny) => z.object({ inner: T })"
                ),
            }]
            .into_iter()
            .collect()
        );

        assert_eq!(
            <Generic::<SerOnly>>::repr_ser(),
            Arg {
                name: String::from("Generic"),
                args: vec![Arg {
                    name: String::from("SerOnly"),
                    args: vec![]
                }]
            }
        );
    }
}
