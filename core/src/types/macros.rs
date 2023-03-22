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
        impl $crate::ast::Node for $T {
            fn export() -> Option<$crate::ast::Export> {
                Some($crate::ast::Export {
                    docs: None,
                    path: $crate::ast::Path::new::<$crate::types::Rs>($name),
                    schema: $crate::ast::Schema::Raw {
                        args: &[],
                        zod: $zod,
                        ts: $ts,
                    },
                })
            }

            fn inline() -> $crate::ast::InlineSchema {
                $crate::ast::InlineSchema::Ref(Self::export().unwrap().path)
            }
        }

        impl $crate::Register for $T {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::register_dependency!(ctx);
            }
        }
    };
}

macro_rules! tuple {
    ( $N: literal, $($i:ident),* ) => {
        Export {
            docs: None,
            path: $crate::ast::Path::new::<$crate::types::Rs>(concat!("Tuple", $N)),
            schema: Schema::Raw {
                args: &[$(GenericArgument::Type(stringify!($i))),*],
                zod: concat!("z.tuple([", $crate::types::macros::join!(", ", $($i),*),"])"),
                ts: concat!("[", $crate::types::macros::join!(", ", $($i),*) ,"]")
            }
        }
    };
}

macro_rules! impl_tuple {
( $N: literal, $($i:ident),* ) => {
        impl<$($i: Node),*> Node for ($($i,)*) {

            fn export() -> Option<Export> {
                Some($crate::types::macros::tuple!($N, $($i),*))
            }

            fn inline() -> InlineSchema {
                InlineSchema::Generic {
                    path: Self::export().unwrap().path,
                    args: vec![$(<$i>::inline()),*],
                }
            }
        }

        impl<$($i: Node),*> Register for ($($i,)*) {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::register_dependency!(ctx, $($i),*);

            }
        }
    }
}

macro_rules! impl_wrapper {
    ($name: literal, $type: ty) => {
        impl<T: Node> Node for $type {
            fn inline() -> InlineSchema {
                T::inline()
            }
        }

        impl<T: Node> Register for $type {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::register_dependency!(ctx, T);
            }
        }
    };
}

macro_rules! impl_generic {
    ({ ty: $ty: ty, name: $name: literal, generics: [$($generics: ident),+], ts: $ts: literal, zod: $zod: literal}) => {
        impl<$($generics: Node),*> Node for $ty {

            fn export() -> Option<Export> {
                Some(Export {
                    docs: None,
                    path: $crate::ast::Path::new::<$crate::types::Rs>($name),
                    schema: Schema::Raw {
                        args: &[$(GenericArgument::Type(stringify!($generics))),+],
                        zod: $zod,
                        ts: $ts
                    },
                })
            }

            fn inline() -> InlineSchema {
                InlineSchema::Ref(Self::export().unwrap().path)
            }
        }

        impl<$($generics: Node),*> Register for $ty {
            fn register(ctx: &mut $crate::DependencyMap)
            where
                Self: 'static,
            {
                $crate::register_dependency!(ctx, $($generics),*);
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
