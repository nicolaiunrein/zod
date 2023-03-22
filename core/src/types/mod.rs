mod external;
pub mod num;
pub use num::*;

pub struct Rs;

impl Namespace for Rs {
    const NAME: &'static str = "Rs";
    const DOCS: Option<&'static str> = Some("Rust types");
}

macro_rules! join {
    ($sep: literal, $first: ident) => {
        stringify!($first)
    };

    ($sep: literal, $first: ident, $($rest: ident),+) => {
        concat!(stringify!($first), $sep, crate::types::join!($sep, $($rest),+))
    };
}

macro_rules! impl_primitive {
    ({ ty: $T:ty, name: $name: literal, ts: $ts: literal, zod: $zod: literal }) => {
        impl crate::ast::Node for $T {
            const PATH: crate::ast::Path = crate::ast::Path {
                ns: <crate::types::Rs as crate::Namespace>::NAME,
                name: $name,
            };

            fn export() -> Option<crate::ast::Export> {
                Some(crate::ast::Export {
                    docs: None,
                    path: Self::PATH,
                    schema: crate::ast::Schema::Raw {
                        args: &[],
                        zod: $zod,
                        ts: $ts,
                    },
                })
            }

            fn inline() -> crate::ast::InlineSchema {
                crate::ast::InlineSchema::Ref(Self::PATH)
            }
        }

        impl crate::Register for $T {
            fn register(ctx: &mut crate::DependencyMap)
            where
                Self: 'static,
            {
                crate::register!(ctx);
            }
        }
    };
}

macro_rules! tuple {
    ( $N: literal, $($i:ident),* ) => {
        Export {
            docs: None,
            path: Self::PATH,
            schema: Schema::Raw {
                args: &[$(GenericArgument::Type(stringify!($i))),*],
                zod: concat!("z.tuple([", crate::types::join!(", ", $($i),*),"])"),
                ts: concat!("[", crate::types::join!(", ", $($i),*) ,"]")
            }
        }
    };
}

macro_rules! impl_tuple {
( $N: literal, $($i:ident),* ) => {
        impl<$($i: Node),*> Node for ($($i,)*) {
            const PATH: Path = Path {
                ns: <crate::types::Rs as crate::Namespace>::NAME,
                name: concat!("Tuple", $N),
            };

            fn export() -> Option<Export> {
                Some(crate::types::tuple!($N, $($i),*))
            }

            fn inline() -> InlineSchema {
                InlineSchema::Generic {
                    path: Self::PATH,
                    args: vec![$(<$i>::inline()),*],
                }
            }
        }

        impl<$($i: Node),*> Register for ($($i,)*) {
            fn register(ctx: &mut crate::DependencyMap)
            where
                Self: 'static,
            {
                crate::register!(ctx, $($i),*);

            }
        }
    }
}

macro_rules! impl_wrapper {
    ($name: literal, $type: ty) => {
        impl<T: Node> Node for $type {
            const PATH: Path = Path {
                ns: <crate::types::Rs as crate::Namespace>::NAME,
                name: $name,
            };

            fn inline() -> InlineSchema {
                T::inline()
            }
        }

        impl<T: Node> Register for $type {
            fn register(ctx: &mut crate::DependencyMap)
            where
                Self: 'static,
            {
                crate::register!(ctx, T);
            }
        }
    };
}

macro_rules! impl_generic {
    ({ ty: $ty: ty, name: $name: literal, generics: [$($generics: ident),+], ts: $ts: literal, zod: $zod: literal}) => {
        impl<$($generics: Node),*> Node for $ty {
            const PATH: Path = Path {
                ns: <crate::types::Rs as crate::Namespace>::NAME,
                name: $name
            };

            fn export() -> Option<Export> {
                Some(Export {
                    docs: None,
                    path: Self::PATH,
                    schema: Schema::Raw {
                        args: &[$(GenericArgument::Type(stringify!($generics))),+],
                        zod: $zod,
                        ts: $ts
                    },
                })
            }

            fn inline() -> InlineSchema {
                InlineSchema::Ref(Self::PATH)
            }
        }

        impl<$($generics: Node),*> Register for $ty {
            fn register(ctx: &mut crate::DependencyMap)
            where
                Self: 'static,
            {
                crate::register!(ctx, $($generics),*);
            }
        }
    }
}

pub(crate) use impl_generic;
pub(crate) use impl_primitive;
pub(crate) use impl_tuple;
pub(crate) use impl_wrapper;
pub(crate) use join;
pub(crate) use tuple;

use crate::Namespace;
// pub(crate) use join;
