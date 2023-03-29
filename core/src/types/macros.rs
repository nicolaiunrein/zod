macro_rules! join {
    ($sep: literal, $first: ident) => {
        stringify!($first)
    };

    ($sep: literal, $first: ident, $($rest: ident),+) => {
        concat!(stringify!($first), $sep, $crate::types::macros::join!($sep, $($rest),+))
    };
}

macro_rules! impl_primitive {
    ({ ty: $T:ty, name: $name: literal, ts: $ts: literal, zod: $zod: literal }) => {
        impl $crate::RequestType for $T {
            const EXPORT: $crate::ast::Export = $crate::ast::Export {
                docs: None,
                path: $crate::ast::Path::new::<$crate::types::Rs>($name),
                schema: $crate::ast::ExportSchema::Raw {
                    args: &[],
                    zod: $zod,
                    ts: $ts,
                },
            };
            const ARGS: &'static [$crate::ast::Ref] = &[];
        }

        impl $crate::RequestTypeVisitor for $T {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::visit_req_dependencies!(ctx);
            }
        }

        impl $crate::ResponseType for $T {
            const EXPORT: $crate::ast::Export = $crate::ast::Export {
                docs: None,
                path: $crate::ast::Path::new::<$crate::types::Rs>($name),
                schema: $crate::ast::ExportSchema::Raw {
                    args: &[],
                    zod: $zod,
                    ts: $ts,
                },
            };

            const ARGS: &'static [$crate::ast::Ref] = &[];
        }

        impl $crate::ResponseTypeVisitor for $T {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::visit_res_dependencies!(ctx);
            }
        }
    };
}

macro_rules! tuple_req {
    ( $N: literal, $($i:ident),* ) => {
        Export {
            docs: None,
            path: $crate::ast::Path::new::<$crate::types::Rs>(concat!("Tuple", $N)),
            schema: $crate::ast::ExportSchema::Raw {
                args: &[$(GenericArgument::Type(stringify!($i))),*],
                zod: concat!("z.tuple([", $crate::types::macros::join!(", ", $($i),*),"])"),
                ts: concat!("[", $crate::types::macros::join!(", ", $($i),*) ,"]")
            },
        }
    };
}

macro_rules! impl_tuple {
( $N: literal, $($i:ident),* ) => {
        impl<$($i: $crate::RequestType),*> $crate::RequestType for ($($i,)*) {
            const EXPORT: $crate::ast::Export = $crate::types::macros::tuple_req!($N, $($i),*);
            const ARGS: &'static [$crate::ast::Ref] = &[$($crate::ast::Ref::new_req::<$i>()),*];
        }

        impl<$($i: $crate::RequestType),*> $crate::RequestTypeVisitor for ($($i,)*) {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::visit_req_dependencies!(ctx, $($i),*);

            }
        }

        impl<$($i: $crate::ResponseType),*> $crate::ResponseType for ($($i,)*) {
            const EXPORT: $crate::ast::Export = $crate::types::macros::tuple_req!($N, $($i),*);
            const ARGS: &'static [$crate::ast::Ref] = &[$($crate::ast::Ref::new_res::<$i>()),*];
        }

        impl<$($i: $crate::ResponseType),*> $crate::ResponseTypeVisitor for ($($i,)*) {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::visit_res_dependencies!(ctx, $($i),*);

            }
        }
    }
}

macro_rules! impl_wrapper {
    ($name: literal, $type: ty) => {
        impl<T: $crate::RequestType> $crate::RequestType for $type {
            const EXPORT: $crate::ast::Export = $crate::ast::Export {
                docs: None,
                path: $crate::ast::Path::new::<$crate::types::Rs>($name),
                schema: $crate::ast::ExportSchema::Newtype($crate::ast::NewtypeSchema::new(
                    &$crate::ast::Ref::new_req::<T>(),
                    false,
                )),
            };
            const ARGS: &'static [$crate::ast::Ref] = &[$crate::ast::Ref::new_req::<T>()];
        }

        impl<T: $crate::RequestType> $crate::RequestTypeVisitor for $type {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::visit_req_dependencies!(ctx, T);
            }
        }

        impl<T: $crate::ResponseType> $crate::ResponseType for $type {
            const EXPORT: $crate::ast::Export = $crate::ast::Export {
                docs: None,
                path: $crate::ast::Path::new::<$crate::types::Rs>($name),
                schema: $crate::ast::ExportSchema::Newtype($crate::ast::NewtypeSchema::new(
                    &$crate::ast::Ref::new_res::<T>(),
                    false,
                )),
            };
            const ARGS: &'static [$crate::ast::Ref] = &[$crate::ast::Ref::new_res::<T>()];
        }

        impl<T: $crate::ResponseType> $crate::ResponseTypeVisitor for $type {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::visit_res_dependencies!(ctx, T);
            }
        }
    };
}

macro_rules! impl_generic {
    ({ ty: $ty: ty, name: $name: literal, generics: [$($generics: ident),+], ts: $ts: literal, zod: $zod: literal}) => {
        impl<$($generics: $crate::RequestType),*> $crate::RequestType for $ty {

            const EXPORT: $crate::ast::Export = $crate::ast::Export {
                    docs: None,
                    path: $crate::ast::Path::new::<$crate::types::Rs>($name),
                    schema: $crate::ast::ExportSchema::Raw {
                        args: &[$(GenericArgument::Type(stringify!($generics))),+],
                        zod: $zod,
                        ts: $ts
                    },
                };

            const ARGS: &'static [$crate::ast::Ref] = &[$($crate::ast::Ref::new_req::<$generics>()),*];
        }


        impl<$($generics: $crate::RequestType),*> $crate::RequestTypeVisitor for $ty {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::visit_req_dependencies!(ctx, $($generics),*);
            }
        }

        impl<$($generics: $crate::ResponseType),*> $crate::ResponseType for $ty {
            const EXPORT: $crate::ast::Export =
                $crate::ast::Export {
                    docs: None,
                    path: $crate::ast::Path::new::<$crate::types::Rs>($name),
                    schema: $crate::ast::ExportSchema::Raw {
                        args: &[$(GenericArgument::Type(stringify!($generics))),+],
                        zod: $zod,
                        ts: $ts
                    },
                };

            const ARGS: &'static [$crate::ast::Ref] = &[$($crate::ast::Ref::new_res::<$generics>()),*];
        }


        impl<$($generics: $crate::ResponseType),*> $crate::ResponseTypeVisitor for $ty {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::visit_res_dependencies!(ctx, $($generics),*);
            }
        }
    }
}

pub(crate) use impl_generic;
pub(crate) use impl_primitive;
pub(crate) use impl_tuple;
pub(crate) use impl_wrapper;
pub(crate) use join;
pub(crate) use tuple_req;
