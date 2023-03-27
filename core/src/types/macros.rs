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
            const AST: $crate::ast::Definition = $crate::ast::Definition::exported(
                $crate::ast::Export {
                    docs: None,
                    path: $crate::ast::Path::new::<$crate::types::Rs>($name),
                    schema: $crate::ast::ExportSchema::Raw {
                        args: &[],
                        zod: $zod,
                        ts: $ts,
                    },
                },
                &[],
            );
        }

        impl $crate::RequestTypeVisitor for $T {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::register_dependencies!(ctx);
            }
        }
    };
}

macro_rules! tuple {
    ( $N: literal, $($i:ident),* ) => {
        Export {
            docs: None,
            path: $crate::ast::Path::new::<$crate::types::Rs>(concat!("Tuple", $N)),
            schema: $crate::ast::ExportSchema::Raw {
                args: &[$(GenericArgument::Type(stringify!($i))),*],
                zod: concat!("z.tuple([", $crate::types::macros::join!(", ", $($i),*),"])"),
                ts: concat!("[", $crate::types::macros::join!(", ", $($i),*) ,"]")
            }
        }
    };
}

macro_rules! impl_tuple {
( $N: literal, $($i:ident),* ) => {
        impl<$($i: RequestType),*> RequestType for ($($i,)*) {

            const AST: $crate::ast::Definition = $crate::ast::Definition::exported(
                $crate::types::macros::tuple!($N, $($i),*),
                &[$(<$i>::AST.inline()),*]
            );
        }

        impl<$($i: RequestType),*> RequestTypeVisitor for ($($i,)*) {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::register_dependencies!(ctx, $($i),*);

            }
        }
    }
}

macro_rules! impl_wrapper {
    ($name: literal, $type: ty) => {
        impl<T: RequestType> RequestType for $type {
            const AST: $crate::ast::Definition = $crate::ast::Definition::inlined(T::AST.inline());
        }

        impl<T: RequestType> RequestTypeVisitor for $type {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::register_dependencies!(ctx, T);
            }
        }
    };
}

macro_rules! impl_generic {
    ({ ty: $ty: ty, name: $name: literal, generics: [$($generics: ident),+], ts: $ts: literal, zod: $zod: literal}) => {
        impl<$($generics: RequestType),*> RequestType for $ty {

            const AST: $crate::ast::Definition = $crate::ast::Definition::exported(
                Export {
                    docs: None,
                    path: $crate::ast::Path::new::<$crate::types::Rs>($name),
                    schema: $crate::ast::ExportSchema::Raw {
                        args: &[$(GenericArgument::Type(stringify!($generics))),+],
                        zod: $zod,
                        ts: $ts
                    },
                },
                &[$($generics::AST.inline()),*]
                );
        }


        impl<$($generics: RequestType),*> RequestTypeVisitor for $ty {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::register_dependencies!(ctx, $($generics),*);
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
