use crate::ast::*;
use crate::Namespace;
use crate::ZodType;

pub struct Rs;

impl Namespace for Rs {
    const NAME: &'static str = "Rs";
    const DOCS: Option<&'static str> = Some("Rust types");
}

macro_rules! impl_primitive {
    ($T:ty, $name: literal, $type: literal, $schema: literal) => {
        impl ZodType for $T {
            const AST: ZodDefinition = ZodDefinition::Literal(Literal {
                ns: Rs::NAME,
                ty: Type {
                    ident: $name,
                    generics: &[],
                },
                ts: $type,
                zod: $schema,
            });
        }

        inventory::submit!(<$T>::AST);
    };
}

macro_rules! impl_tuple {
    ( $N: literal, $($i:ident),* ) => {
        impl<$($i: ZodType),*> ZodType for ($($i,)*) {
            const AST: ZodDefinition = tuple!($N, $($i),*);
        }

    };
}

macro_rules! tuple {
    ( $N: literal, $($i:ident),* ) => {

        {

            const AST: ZodDefinition = ZodDefinition::Literal(Literal {
                ns: Rs::NAME,
                ty: Type {
                    ident: concat!("Tuple", $N),
                    generics: &[]
                },
                ts: concat!("export type Tuple", $N, "<",std::stringify!($($i),*) ,">",  " = [", std::stringify!($($i),*), "];"),
                zod: concat!("export const Tuple", $N, " = (", $(std::stringify!($i: z.ZodTypeAny,)),*  ,") => z.tuple([", $(std::stringify!(z.lazy(() => $i),)),*, "])"),
            });

            inventory::submit!(AST);
            AST
        }
    };
}

macro_rules! impl_wrapper {
    ($($t:tt)*) => {
        $($t)* {
            const AST: ZodDefinition = T::AST;
        }
    };
}

impl_primitive!(String, "String", "string", "z.string()");
impl_primitive!(&str, "String", "string", "z.string()");

impl_primitive!(
    u8,
    "U8",
    "number",
    "z.number().finite().int().nonnegative().lte(255)"
);
impl_primitive!(
    u16,
    "U16",
    "number",
    "z.number().finite().int().nonnegative().lte(65535)"
);
impl_primitive!(
    u32,
    "U32",
    "number",
    "z.number().finite().int().nonnegative().lte(4294967295)"
);
impl_primitive!(
    u64,
    "U64",
    "number",
    "z.number().finite().int().nonnegative().lte(18446744073709551615)"
);
impl_primitive!(
    u128,
    "U128",
    "number",
    "z.number().finite().int().nonnegative().lte(340282366920938463463374607431768211455)"
);
impl_primitive!(
    usize,
    "Usize",
    "number",
    "z.number().finite().int().nonnegative()"
);

impl_primitive!(
    i8,
    "I8",
    "number",
    "z.number().finite().int().lte(127).gte(-128)"
);
impl_primitive!(
    i16,
    "I16",
    "number",
    "z.number().finite().int().lte(32767).gte(-32768)"
);
impl_primitive!(
    i32,
    "I32",
    "number",
    "z.number().finite().int().lte(2147483647).gte(-2147483648)"
);
impl_primitive!(
    i64,
    "I64",
    "number",
    "z.number().finite().int().lte(9223372036854775807).gte(-9223372036854775808)"
);
impl_primitive!(i128, "I128", "number", "z.number().finite().int().lte(170141183460469231731687303715884105727).gte(-170141183460469231731687303715884105728)");
impl_primitive!(isize, "Isize", "number", "z.number().finite().int()");

impl_primitive!(f32, "F32", "number", "z.number()");
impl_primitive!(f64, "F64", "number", "z.number()");

impl_primitive!(bool, "Bool", "boolean", "z.boolean()");
impl_primitive!(char, "Char", "string", "z.string().length(1)");
impl_primitive!((), "Unit", "null", "z.null()");

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

impl_wrapper!(impl<T: ZodType> ZodType for Box<T>);
impl_wrapper!(impl<T: ZodType> ZodType for std::sync::Arc<T>);
impl_wrapper!(impl<T: ZodType> ZodType for std::rc::Rc<T>);
impl_wrapper!(impl<T: ZodType + ToOwned> ZodType for std::borrow::Cow<'static, T>);
impl_wrapper!(impl<T: ZodType> ZodType for std::cell::Cell<T>);
impl_wrapper!(impl<T: ZodType> ZodType for std::cell::RefCell<T>);
impl_wrapper!(impl<T: ZodType> ZodType for std::sync::Mutex<T>);
impl_wrapper!(impl<T: ZodType> ZodType for std::sync::Weak<T>);
impl_wrapper!(impl<T: ZodType> ZodType for std::marker::PhantomData<T>);

impl<T: ZodType> ZodType for Vec<T> {
    const AST: ZodDefinition = ZodDefinition::Literal(Literal {
        ns: Rs::NAME,
        ty: Type {
            ident: "Vec",
            generics: &[Generic::Type { ident: "T" }],
        },
        ts: "export type Vec<T> = T[];",
        zod: "export const Vec = (T: z.ZodTypeAny) => z.array(z.lazy(() => T))",
    });
}

impl<const N: usize, T: ZodType> ZodType for [T; N] {
    const AST: ZodDefinition = ZodDefinition::Literal(Literal {
            ns: Rs::NAME,
            ty: Type{
                ident: "Array",
                generics: &[Generic::Type {ident: "T"}]
            },
            ts: "
        export type Array<N extends number, T, TObj = [T, ...T[]]> = Pick<TObj, Exclude<keyof TObj, 'splice' | 'push' | 'pop' | 'shift' |  'unshift'>>
          & {
            readonly length: N 
            [ I : number ] : T
            [Symbol.iterator]: () => IterableIterator<T>   
          }
            ",
            zod:
                "export const Array = (N: number, T: z.ZodTypeAny) => z.array(z.lazy(() => T)).length(N)",
    });
}

const HASH_SET_AST: ZodDefinition = ZodDefinition::Literal(Literal {
    ns: Rs::NAME,
    ty: Type {
        ident: "HashSet",
        generics: &[Generic::Type { ident: "T" }],
    },
    ts: "export type HashSet<T> = Set<T>;",
    zod: "export const HashSet = (T: z.ZodTypeAny) => z.set(z.lazy(() => T))",
});

impl<T: ZodType> ZodType for std::collections::HashSet<T> {
    const AST: ZodDefinition = HASH_SET_AST;
}
inventory::submit!(HASH_SET_AST);

const BTREE_SET_AST: ZodDefinition = ZodDefinition::Literal(Literal {
    ns: Rs::NAME,
    ty: Type {
        ident: "BTreeSet",
        generics: &[Generic::Type { ident: "T" }],
    },
    ts: "export type BTreeSet<T> = Set<T>;",
    zod: "export const BTreeSet = (T: z.ZodTypeAny) => z.set(z.lazy(() => T))",
});

impl<T: ZodType> ZodType for std::collections::BTreeSet<T> {
    const AST: ZodDefinition = BTREE_SET_AST;
}

inventory::submit!(BTREE_SET_AST);

const HASH_MAP_AST: ZodDefinition = ZodDefinition::Literal(Literal {
            ns: Rs::NAME,
            ty: Type {
                ident: "HashMap",
                generics: &[
                    Generic::Type {ident: "K"},
                    Generic::Type {ident: "V"},
                ]
            },
            ts: "export type HashMap<K, V> = Map<K, V>;",
            zod: "export const HashMap = (K: z.ZodTypeAny, V: z.ZodTypeAny) => z.map(z.lazy(() => K), z.lazy(() => V));",
    });

impl<K: ZodType, V: ZodType> ZodType for std::collections::HashMap<K, V> {
    const AST: ZodDefinition = HASH_MAP_AST;
}

inventory::submit!(HASH_MAP_AST);

const BTREE_MAP_AST: ZodDefinition = ZodDefinition::Literal(Literal {
            ns: Rs::NAME,
            ty: Type {
                ident: "BTreeMap",
                generics: &[
                    Generic::Type {ident: "K"},
                    Generic::Type {ident: "V"},
                ]
            },
            ts: "export type BTreeMap<K, V> = Map<K, V>;",
            zod: "export const BTreeMap = (K: z.ZodTypeAny, V: z.ZodTypeAny) => z.map(z.lazy(() => K), z.lazy(() => V));",
    });

impl<K: ZodType, V: ZodType> ZodType for std::collections::BTreeMap<K, V> {
    const AST: ZodDefinition = BTREE_MAP_AST;
}

inventory::submit!(BTREE_MAP_AST);

const OPTION_AST: ZodDefinition = ZodDefinition::Struct(Struct {
    ns: Rs::NAME,
    ty: Type {
        ident: "Option",
        generics: &[Generic::Type { ident: "T" }],
    },
    fields: StructFields::Transparent {
        value: FieldValue::Generic(Generic::Type { ident: "T" }),
        optional: true,
    },
});

impl<T: ZodType> ZodType for Option<T> {
    const AST: ZodDefinition = OPTION_AST;
}

inventory::submit!(OPTION_AST);

const RESULT_AST: ZodDefinition = ZodDefinition::Literal(Literal {
            ns: Rs::NAME,
            ty: Type {
                ident: "Result",
                generics: &[
                    Generic::Type {ident: "T"},
                    Generic::Type {ident: "E"},
                ]

            },
            ts: "export type Result<T, E> = { Ok: T } | { Err: E };",
            zod: "export const Result = (T: z.ZodTypeAny, E: z.ZodTypeError) => z.union([z.object({ Ok: z.lazy(() => T) }), z.object({ Err: z.lazy(() => E) })])"
    });

impl<T: ZodType, E: ZodType> ZodType for Result<T, E> {
    const AST: ZodDefinition = RESULT_AST;
}

inventory::submit!(RESULT_AST);

impl_primitive!(
    std::net::Ipv4Addr,
    "Ipv4Addr",
    "string",
    "z.string().ip({ version: \"v4\" })"
);

impl_primitive!(
    std::net::Ipv6Addr,
    "Ipv6Addr",
    "string",
    "z.string().ip({ version: \"v6\" })"
);

impl_primitive!(std::net::IpAddr, "IpAddr", "string", "z.string().ip()");

#[cfg(feature = "smol_str")]
impl_primitive!(smol_str::SmolStr, "String", "string", "z.string()");

#[cfg(feature = "ordered-float")]
impl_primitive!(ordered_float::NotNan<f32>, "F32", "number", "z.number()");

#[cfg(feature = "ordered-float")]
impl_primitive!(ordered_float::NotNan<f64>, "F64", "number", "z.number()");

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn option_ok() {
        assert_eq!(
            Option::<String>::AST.to_ts_string(),
            "export type Option<T> = T | undefined;"
        );
        assert_eq!(
            Option::<String>::AST.to_zod_string(),
            "export const Option = (T: z.ZodTypeAny) => z.lazy(() => T.optional());"
        );
    }

    #[test]
    fn inventory() {
        let items = inventory::iter::<ZodDefinition>();

        let item_names: BTreeSet<_> = items
            .filter_map(|item| {
                if item.ns() == Rs::NAME {
                    Some(item.ty().ident)
                } else {
                    None
                }
            })
            .collect();

        assert_eq!(
            &item_names,
            &[
                "BTreeMap",
                "BTreeSet",
                "HashMap",
                "HashSet",
                "Option",
                "Result",
                "Tuple1",
                "Tuple10",
                "Tuple11",
                "Tuple12",
                "Tuple2",
                "Tuple3",
                "Tuple4",
                "Tuple5",
                "Tuple6",
                "Tuple7",
                "Tuple8",
                "Tuple9",
                <&str>::AST.ty().ident,
                <()>::AST.ty().ident,
                String::AST.ty().ident,
                bool::AST.ty().ident,
                char::AST.ty().ident,
                f32::AST.ty().ident,
                f64::AST.ty().ident,
                i128::AST.ty().ident,
                i16::AST.ty().ident,
                i32::AST.ty().ident,
                i64::AST.ty().ident,
                i8::AST.ty().ident,
                isize::AST.ty().ident,
                std::net::IpAddr::AST.ty().ident,
                std::net::Ipv4Addr::AST.ty().ident,
                std::net::Ipv6Addr::AST.ty().ident,
                u128::AST.ty().ident,
                u16::AST.ty().ident,
                u32::AST.ty().ident,
                u64::AST.ty().ident,
                u8::AST.ty().ident,
                usize::AST.ty().ident,
            ]
            .into_iter()
            .collect()
        )
    }
}
