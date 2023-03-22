use crate::num::Usize;
use crate::Namespace;

use super::{Export, GenericArgument, InlineSchema, Node, Path, Register, Schema};

const ARRAY_SCHEMA: &str = r#"
Pick<
  TObj,
  Exclude<keyof TObj, "splice" | "push" | "pop" | "shift" | "unshift">
> & {
  readonly length: N;
  [I: number]: T;
  [Symbol.iterator]: () => IterableIterator<T>;
};
"#;

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
        concat!(stringify!($first), $sep, join!($sep, $($rest),+))
    };
}

macro_rules! impl_primitive {
    ({ ty: $T:ty, name: $name: literal, ts: $ts: literal, zod: $zod: literal }) => {
        impl Node for $T {
            const PATH: Path = Path {
                ns: Rs::NAME,
                name: $name,
            };

            fn export() -> Option<Export> {
                Some(Export {
                    docs: None,
                    path: Self::PATH,
                    schema: Schema::Raw {
                        args: &[],
                        zod: $zod,
                        ts: $ts,
                    },
                })
            }

            fn inline() -> InlineSchema {
                InlineSchema::Ref(Self::PATH)
            }
        }

        impl Register for $T {
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
                zod: concat!("z.tuple([", join!(", ", $($i),*),"])"),
                ts: concat!("[", join!(", ", $($i),*) ,"]")
            }
        }
    };
}

macro_rules! impl_tuple {
( $N: literal, $($i:ident),* ) => {
        impl<$($i: Node),*> Node for ($($i,)*) {
            const PATH: Path = Path {
                ns: Rs::NAME,
                name: concat!("Tuple", $N),
            };

            fn export() -> Option<Export> {
                Some(tuple!($N, $($i),*))
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
                ns: Rs::NAME,
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
                ns: Rs::NAME,
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

impl_tuple!(1, T1);
impl_tuple!(2, T1, T2);
impl_tuple!(3, T1, T2, T3);
impl_tuple!(4, T1, T2, T3, T4);
impl_tuple!(5, T1, T2, T3, T4, T5);
impl_tuple!(6, T1, T2, T3, T4, T5, T6);
impl_tuple!(7, T1, T2, T3, T4, T5, T6, T7);
impl_tuple!(8, T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple!(9, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple!(10, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_tuple!(11, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_tuple!(12, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);

impl_primitive!({
    ty: String,
    name: "String",
    ts: "string",
    zod: "z.string()"
});

impl_primitive!({
    ty: &str,
    name: "String",
    ts: "string",
    zod: "z.string()"
});

impl_primitive!({
    ty: u8,
    name: "U8",
    ts: "number",
    zod: "z.number().finite().int().nonnegative().lte(255)"
});

impl_primitive!({
    ty: u16,
    name: "U16",
    ts: "number",
    zod: "z.number().finite().int().nonnegative().lte(65535)"
});

impl_primitive!({
    ty: u32,
    name: "U32",
    ts: "number",
    zod: "z.number().finite().int().nonnegative().lte(4294967295)"
});

impl_primitive!({
    ty: crate::num::U64,
    name: "U64",
    ts: "number",
    zod: "z.bigint().nonnegative().lt(2n ** 64n)"
});

impl_primitive!({
    ty: crate::num::U128,
    name: "U128",
    ts: "number",
    zod: "z.bigint().nonnegative().lt(2n ** 128n)"
});

#[cfg(target_pointer_width = "64")]
impl_primitive!({
    ty: crate::num::Usize,
    name: "Usize",
    ts: "BigInt",
    zod: "z.bigint().nonnegative().lt(2n ** 64n)"
});

#[cfg(target_pointer_width = "32")]
impl_primitive!({
    ty: crate::num::Usize,
    name: "Usize",
    ts: "BigInt",
    zod: "z.bigint().nonnegative().lt(2n ** 32n)"
});

#[cfg(target_pointer_width = "16")]
impl_primitive!({
    ty: crate::num::Usize,
    name: "Usize",
    ts: "BigInt",
    zod: "z.bigint().nonnegative().lt(2n ** 16n)"
});

impl_primitive!({
    ty: i8,
    name: "I8",
    ts: "number",
    zod: "z.number().finite().int().lte(127).gte(-128)"
});

impl_primitive!({
    ty: i16,
    name: "I16",
    ts: "number",
    zod: "z.number().finite().int().lte(32767).gte(-32768)"
});

impl_primitive!({
    ty: i32,
    name: "I32",
    ts: "number",
    zod: "z.number().finite().int().lte(2147483647).gte(-2147483648)"
});

impl_primitive!({
    ty: crate::num::I64,
    name: "I64",
    ts: "number",
    zod: "z.bigint().gte(-(2n ** 63n)).lt(2n ** 63n)"
});

impl_primitive!({
    ty: crate::num::I128,
    name: "I128",
    ts: "number",
    zod: "z.bigint().gte(-(2n ** 127n)).lt(2n ** 127n)"
});

#[cfg(target_pointer_width = "64")]
impl_primitive!({
    ty: crate::num::Isize,
    name: "Isize",
    ts: "number",
    zod: "z.bigint().gte(-(2n ** 63n)).lt(2n ** 63n)"
});

#[cfg(target_pointer_width = "32")]
impl_primitive!({
    ty: crate::num::Isize,
    name: "Isize",
    ts: "number",
    zod: "z.bigint().gte(-(2n ** 31n)).lt(2n ** 31n)"
});

#[cfg(target_pointer_width = "16")]
impl_primitive!({
    ty: crate::num::Isize,
    name: "Isize",
    ts: "number",
    zod: "z.bigint().gte(-(2n ** 15n)).lt(2n ** 15n)"
});

impl_primitive!({
    ty: f32,
    name: "F32",
    ts: "number",
    zod: "z.number()"
});

impl_primitive!({
    ty: f64,
    name: "F64",
    ts: "number",
    zod: "z.number()"
});

impl_primitive!({
    ty: bool,
    name: "Bool",
    ts: "boolean",
    zod: "z.boolean()"
});
impl_primitive!({
    ty: char,
    name: "Char",
    ts: "string",
    zod: "z.string().length(1)"
});

impl_primitive!({
    ty: (),
    name: "Unit",
    ts: "null",
    zod: "z.null()"
});

impl_primitive!({
    ty: std::net::Ipv4Addr,
    name: "Ipv4Addr",
    ts: "string",
    zod: "z.string().ip({ version: \"v4\" })"
});

impl_primitive!({
    ty: std::net::Ipv6Addr,
    name: "Ipv6Addr",
    ts: "string",
    zod: "z.string().ip({ version: \"v6\" })"
});

impl_primitive!({
    ty: std::net::IpAddr,
    name: "IpAddr",
    ts: "string",
    zod: "z.string().ip()"
});

impl_wrapper!("Box", Box<T>);
impl_wrapper!("Arc", std::sync::Arc<T>);
impl_wrapper!("Rc", std::rc::Rc<T>);
impl_wrapper!("Cell", std::cell::Cell<T>);
impl_wrapper!("RefCell", std::cell::RefCell<T>);
impl_wrapper!("Mutex", std::sync::Mutex<T>);
impl_wrapper!("Weak", std::sync::Weak<T>);

impl_generic!({
    ty: Vec<T>,
    name: "Vec",
    generics: [T],
    ts: "T[]",
    zod: "z.array(T)"
});

impl_generic!({
    ty: std::collections::HashSet<T>,
    name: "HashSet",
    generics: [T],
    ts: "Set<T>",
    zod: "z.set(T)"
});

impl_generic!({
    ty: std::collections::BTreeSet<T>,
    name: "HashSet",
    generics: [T],
    ts: "Set<T>",
    zod: "z.set(T)"
});

impl_generic!({
    ty: std::collections::HashMap<K, V>,
    name: "HashMap",
    generics: [K, V],
    ts: "Map<K, V>",
    zod: "z.map(K, V)"
});

impl_generic!({
    ty: std::collections::BTreeMap<K, V>,
    name: "BTreeMap",
    generics: [K, V],
    ts: "Map<K, V>",
    zod: "z.map(K, V)"
});

impl_generic!({
    ty: Result<T, E>,
    name: "Result",
    generics: [T, E],
    ts: "{ Ok: T } | { Err: E }",
    zod: "z.union([z.object({ Ok: T }), z.object({ Err: E })])"
});

impl_generic!({
    ty: Option<T>,
    name: "Option",
    generics: [T],
    ts: "T | undefined",
    zod: "T.optional()"
});

impl<T: Node + ToOwned> Node for std::borrow::Cow<'static, T> {
    const PATH: Path = Path {
        ns: Rs::NAME,
        name: "Cow",
    };

    fn inline() -> InlineSchema {
        T::inline()
    }
}

impl<T: Node + ToOwned> Register for std::borrow::Cow<'static, T> {
    fn register(ctx: &mut crate::DependencyMap)
    where
        Self: 'static,
    {
        crate::register!(ctx, T);
    }
}

impl<const N: usize, T: Node> Node for [T; N] {
    const PATH: Path = Path {
        ns: "Rs",
        name: "Array",
    };

    fn export() -> Option<Export> {
        Some(Export {
            docs: None,
            path: Self::PATH,
            schema: Schema::Raw {
                args: &[
                    GenericArgument::Type("T"),
                    GenericArgument::Const {
                        name: "N",
                        path: Usize::PATH,
                    },
                    GenericArgument::Assign {
                        name: "TObj",
                        value: "[T, ...T[]]",
                    },
                ],
                zod: "z.array(T).length(N)",
                ts: ARRAY_SCHEMA,
            },
        })
    }

    fn inline() -> InlineSchema {
        InlineSchema::Ref(Self::PATH)
    }
}

impl<const N: usize, T: Node> Register for [T; N] {
    fn register(ctx: &mut crate::DependencyMap)
    where
        Self: 'static,
    {
        crate::register!(ctx, T);
    }
}

#[cfg(feature = "smol_str")]
impl_primitive!({
    ty: smol_str::SmolStr,
    name: "String",
    ts: "string",
    zod: "z.string()"
});

#[cfg(feature = "ordered-float")]
impl_primitive!({
    ty: ordered_float::NotNan<f32>,
    name: "F32",
    ts: "number",
    zod: "z.number()"
});

#[cfg(feature = "ordered-float")]
impl_primitive!({
    ty: ordered_float::NotNan<f64>,
    name: "F64",
    ts: "number",
    zod: "z.number()"
});

#[cfg(test)]
mod test {
    use crate::ast::Formatter;
    use crate::num::Usize;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn string_ok() {
        let export = <String>::export();
        let expected_zod_export = "export const String = z.lazy(() => z.string());";
        let expected_ts_export = "export type String = string;";

        assert_eq!(
            export.as_ref().unwrap().to_zod_string(),
            expected_zod_export
        );

        assert_eq!(export.as_ref().unwrap().to_ts_string(), expected_ts_export);
    }

    #[test]
    fn option_ok() {
        let export = <Option<String>>::export();
        let expected_zod_export =
            "export const Option = z.lazy(() => (T: z.ZodTypeAny) => T.optional());";
        let expected_ts_export = "export type Option<T> = T | undefined;";

        assert_eq!(
            export.as_ref().unwrap().to_zod_string(),
            expected_zod_export
        );

        assert_eq!(export.as_ref().unwrap().to_ts_string(), expected_ts_export);
    }

    #[test]
    fn array_ok() {
        let export = <[String; 5]>::export();
        assert_eq!(
            export.as_ref().unwrap().to_zod_string(),
            "export const Array = z.lazy(() => (T: z.ZodTypeAny, N: Rs.Usize) => z.array(T).length(N));"
        );

        assert_eq!(
            export.as_ref().unwrap().to_ts_string(),
            format!(
                "export type Array<T, N extends Rs.Usize, TObj = [T, ...T[]]> = {};",
                ARRAY_SCHEMA
            )
        )
    }

    #[test]
    fn join() {
        assert_eq!(join!("_", a, b, c), "a_b_c");
    }
    #[test]
    fn tuple_ok() {
        let export = <(String, Usize)>::export();

        assert_eq!(export.as_ref().unwrap().to_zod_string(), "export const Tuple2 = z.lazy(() => (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.tuple([T1, T2]));");
        assert_eq!(
            export.as_ref().unwrap().to_ts_string(),
            "export type Tuple2<T1, T2> = [T1, T2];"
        );
    }

    #[test]
    fn wrapper_ok() {
        let export = <Box<String>>::export();
        let inline = <Box<String>>::inline();

        assert!(export.is_none());

        assert_eq!(inline.to_zod_string(), "Rs.String");
        assert_eq!(inline.to_ts_string(), "Rs.String");
    }

    #[test]
    fn vec_ok() {
        let export = <Vec<String>>::export();

        assert_eq!(
            export.as_ref().unwrap().to_zod_string(),
            "export const Vec = z.lazy(() => (T: z.ZodTypeAny) => z.array(T));"
        );
        assert_eq!(
            export.as_ref().unwrap().to_ts_string(),
            "export type Vec<T> = T[];"
        );
    }

    #[test]
    fn bigints_ok() {
        let json = serde_json::to_value(&Usize(123123)).unwrap();
        assert_eq!(json, serde_json::json!("123123"));

        let num: Usize = serde_json::from_value(json).unwrap();
        assert_eq!(num, 123123);

        let export = Usize::export().unwrap();
        assert_eq!(
            export.to_zod_string(),
            "export const Usize = z.lazy(() => z.bigint().nonnegative().lt(2n ** 64n));"
        );
    }
}
