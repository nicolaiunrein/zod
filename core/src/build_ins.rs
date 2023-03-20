use crate::ast::*;
use crate::DependencyMap;
use crate::DependencyRegistration;
use crate::Namespace;
use crate::ZodType;
use phf::phf_map;

pub struct Rs;

pub struct RsRegistry;

impl Namespace for Rs {
    const NAME: &'static str = "Rs";
    const DOCS: Option<&'static str> = Some("Rust types");
    type Registry = RsRegistry;
}

// macro_rules! impl_primitive {
// ($T:ty, $name: literal, $ts_type: literal, $zod: literal) => {
// impl ZodType for $T {
// const AST: ZodExport = ZodExport {
// docs: None,
// def: ZodDefinition::Literal(Literal {
// ty: TypeDef {
// ns: Rs::NAME,
// ident: $name,
// generics: &[],
// },
// ts: concat!("type ", $name, " = ", $ts_type, ";"),
// zod: concat!("const ", $name, " = ", $zod, ";"),
// }),
// };
// }

// impl DependencyRegistration for $T {
// fn register_dependencies(cx: &mut DependencyMap)
// where
// Self: 'static,
// {
// cx.add::<Self>();
// }
// }
// };
// }

// macro_rules! impl_tuple {
// ( $N: literal, $($i:ident),* ) => {
// impl<$($i: ZodType),*> ZodType for ($($i,)*) {
// const AST: ZodExport = tuple!($N, $($i),*);
// }

// impl<$($i: ZodType),*> DependencyRegistration for ($($i,)*) {
// fn register_dependencies(cx: &mut DependencyMap)
// where
// Self: 'static,
// {
// if cx.add::<Self>(){
// $(<$i>::register_dependencies(cx);)*
// }
// }

// }

// };
// }

// macro_rules! tuple {
// ( $N: literal, $($i:ident),* ) => {

// {
// ZodExport {
// docs: None,
// def: ZodDefinition::Struct(Struct {
// ty: TypeDef {
// ns: Rs::NAME,
// ident: concat!("Tuple", $N),
// generics: &[
// $(Generic::new_for::<$i>(stringify!($i))),*
// ]
// },
// fields: StructFields::Tuple(&[
// $(TupleField{
// optional: false,
// value: FieldValue::new_for::<$i>(&phf_map! { 0_u64 => stringify!($i) })
// }),*
// ])
// })}
// }
// };
// }

// macro_rules! impl_wrapper {
// ($type:ty, $($t:tt)* ) => {
// $($t)* ZodType for $type {
// const AST: ZodExport = T::AST;
// // const AST: ZodExport = ZodExport {
// // docs: None,
// // def: ZodDefinition::Struct(Struct {
// // ty: T::AST.def.ty(),
// // fields:

// // })

// // }
// }

// $($t)* DependencyRegistration for $type {

// fn register_dependencies(cx: &mut DependencyMap)
// where
// Self: 'static,
// {
// if cx.add::<Self>() {
// T::register_dependencies(cx);
// }
// }
// }
// };
// }

// impl_primitive!(String, "String", "string", "z.string()");
// impl_primitive!(&str, "String", "string", "z.string()");

// impl_primitive!(
// u8,
// "U8",
// "number",
// "z.number().finite().int().nonnegative().lte(255)"
// );
// impl_primitive!(
// u16,
// "U16",
// "number",
// "z.number().finite().int().nonnegative().lte(65535)"
// );
// impl_primitive!(
// u32,
// "U32",
// "number",
// "z.number().finite().int().nonnegative().lte(4294967295)"
// );
// impl_primitive!(
// u64,
// "U64",
// "number",
// "z.number().finite().int().nonnegative().lte(18446744073709551615)"
// );
// impl_primitive!(
// u128,
// "U128",
// "number",
// "z.number().finite().int().nonnegative().lte(340282366920938463463374607431768211455)"
// );
// impl_primitive!(
// usize,
// "Usize",
// "number",
// "z.number().finite().int().nonnegative()"
// );

// impl_primitive!(
// i8,
// "I8",
// "number",
// "z.number().finite().int().lte(127).gte(-128)"
// );
// impl_primitive!(
// i16,
// "I16",
// "number",
// "z.number().finite().int().lte(32767).gte(-32768)"
// );
// impl_primitive!(
// i32,
// "I32",
// "number",
// "z.number().finite().int().lte(2147483647).gte(-2147483648)"
// );
// impl_primitive!(
// i64,
// "I64",
// "number",
// "z.number().finite().int().lte(9223372036854775807).gte(-9223372036854775808)"
// );
// impl_primitive!(i128, "I128", "number", "z.number().finite().int().lte(170141183460469231731687303715884105727).gte(-170141183460469231731687303715884105728)");
// impl_primitive!(isize, "Isize", "number", "z.number().finite().int()");

// impl_primitive!(f32, "F32", "number", "z.number()");
// impl_primitive!(f64, "F64", "number", "z.number()");

// impl_primitive!(bool, "Bool", "boolean", "z.boolean()");
// impl_primitive!(char, "Char", "string", "z.string().length(1)");
// impl_primitive!((), "Unit", "null", "z.null()");

// impl_tuple!(1, T1);
// impl_tuple!(2, T1, T2);
// impl_tuple!(3, T1, T2, T3);
// impl_tuple!(4, T1, T2, T3, T4);
// impl_tuple!(5, T1, T2, T3, T4, T5);
// impl_tuple!(6, T1, T2, T3, T4, T5, T6);
// impl_tuple!(7, T1, T2, T3, T4, T5, T6, T7);
// impl_tuple!(8, T1, T2, T3, T4, T5, T6, T7, T8);
// impl_tuple!(9, T1, T2, T3, T4, T5, T6, T7, T8, T9);
// impl_tuple!(10, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
// impl_tuple!(11, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
// impl_tuple!(12, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);

// impl_wrapper!(Box<T>, impl<T: ZodType>);
// impl_wrapper!(std::sync::Arc<T>, impl <T: ZodType>);
// impl_wrapper!(std::rc::Rc<T>, impl<T: ZodType>);
// impl_wrapper!(std::borrow::Cow<'static, T>, impl<T: ZodType + ToOwned>);
// impl_wrapper!(std::cell::Cell<T>, impl<T: ZodType>);
// impl_wrapper!(std::cell::RefCell<T>, impl<T: ZodType>);
// impl_wrapper!(std::sync::Mutex<T>, impl<T: ZodType>);
// impl_wrapper!(std::sync::Weak<T>, impl<T: ZodType>);
// impl_wrapper!(std::marker::PhantomData<T>, impl<T: ZodType>);

// impl<T: ZodType> ZodType for Vec<T> {
// const AST: ZodExport = ZodExport {
// docs: None,
// def: ZodDefinition::Literal(Literal {
// ty: TypeDef {
// ns: Rs::NAME,
// ident: "Vec",
// generics: &[Generic::new_for::<T>("T")],
// },
// ts: "export type Vec<T> = T[];",
// zod: "export const Vec = (T: z.ZodTypeAny) => z.array(z.lazy(() => T))",
// }),
// };
// }

// impl<T: ZodType> DependencyRegistration for Vec<T> {
// fn register_dependencies(cx: &mut DependencyMap)
// where
// Self: 'static,
// {
// if cx.add::<Self>() {
// T::register_dependencies(cx);
// }
// }
// }

// impl<const N: usize, T: ZodType> ZodType for [T; N] {
// const AST: ZodExport = ZodExport{
// docs: None,
// def: ZodDefinition::Literal(Literal {
// ty: TypeDef{
// ns: Rs::NAME,
// ident: "Array",
// //todo
// generics: &[Generic::new_for::<T>("T")],
// },
// ts: "
// export type Array<N extends number, T, TObj = [T, ...T[]]> = Pick<TObj, Exclude<keyof TObj, 'splice' | 'push' | 'pop' | 'shift' |  'unshift'>>
// & {
// readonly length: N
// [ I : number ] : T
// [Symbol.iterator]: () => IterableIterator<T>
// }
// ",
// zod:
// "export const Array = (N: number, T: z.ZodTypeAny) => z.array(z.lazy(() => T)).length(N)",
// })};
// }

// impl<const N: usize, T: ZodType> DependencyRegistration for [T; N] {
// fn register_dependencies(cx: &mut DependencyMap)
// where
// Self: 'static,
// {
// if cx.add::<Self>() {
// T::register_dependencies(cx);
// }
// }
// }

// impl<T: ZodType> ZodType for std::collections::HashSet<T> {
// const AST: ZodExport = ZodExport {
// docs: None,
// def: ZodDefinition::Literal(Literal {
// ty: TypeDef {
// ns: Rs::NAME,
// ident: "HashSet",
// generics: &[Generic::new_for::<T>("T")],
// },
// ts: "export type HashSet<T> = Set<T>;",
// zod: "export const HashSet = (T: z.ZodTypeAny) => z.set(z.lazy(() => T))",
// }),
// };
// }
// impl<T: ZodType> DependencyRegistration for std::collections::HashSet<T> {
// fn register_dependencies(cx: &mut DependencyMap)
// where
// Self: 'static,
// {
// if cx.add::<Self>() {
// T::register_dependencies(cx);
// }
// }
// }

// impl<T: ZodType> ZodType for std::collections::BTreeSet<T> {
// const AST: ZodExport = ZodExport {
// docs: None,
// def: ZodDefinition::Literal(Literal {
// ty: TypeDef {
// ns: Rs::NAME,
// ident: "BTreeSet",
// generics: &[Generic::new_for::<T>("T")],
// },
// ts: "export type BTreeSet<T> = Set<T>;",
// zod: "export const BTreeSet = (T: z.ZodTypeAny) => z.set(z.lazy(() => T))",
// }),
// };
// }

// impl<T: ZodType> DependencyRegistration for std::collections::BTreeSet<T> {
// fn register_dependencies(cx: &mut DependencyMap)
// where
// Self: 'static,
// {
// if cx.add::<Self>() {
// T::register_dependencies(cx);
// }
// }
// }

// impl<K: ZodType, V: ZodType> ZodType for std::collections::HashMap<K, V> {
// const AST: ZodExport =  ZodExport{
// docs: None,
// def: ZodDefinition::Literal(Literal {
// ty: TypeDef {
// ns: Rs::NAME,
// ident: "HashMap",
// generics: &[
// Generic::new_for::<K>("K"),
// Generic::new_for::<V>("V")
// ],
// },
// ts: "export type HashMap<K, V> = Map<K, V>;",
// zod: "export const HashMap = (K: z.ZodTypeAny, V: z.ZodTypeAny) => z.map(z.lazy(() => K), z.lazy(() => V));",
// })};
// }

// impl<K: ZodType, V: ZodType> DependencyRegistration for std::collections::HashMap<K, V> {
// fn register_dependencies(cx: &mut DependencyMap)
// where
// Self: 'static,
// {
// if cx.add::<Self>() {
// K::register_dependencies(cx);
// V::register_dependencies(cx);
// }
// }
// }

// // impl<K: ZodType, V: ZodType> ZodType for std::collections::BTreeMap<K, V> {
// //     fn AST() -> crate::AST::ZodExport {
// //         ZodExport{
// //     docs: None,def:ZodDefinition::Literal(Literal {
// //             ns: Rs::NAME,
// //             ty: Type {
// //                 ident: "BTreeMap",
// //                 generics: &[
// //                     Generic::Type {ident: "K"},
// //                     Generic::Type {ident: "V"},
// //                 ]
// //             },
// //             ts: "export type BTreeMap<K, V> = Map<K, V>;",
// //             zod: "export const BTreeMap = (K: z.ZodTypeAny, V: z.ZodTypeAny) => z.map(z.lazy(() => K), z.lazy(() => V));",
// //     })}
// //     }
// //
// //     const INLINED: Inlined = Inlined {
// //         ns: Self::AST().ns(),
// //         name: Self::AST().name(),
// //         params: &[<K>::INLINED, <V>::INLINED],
// //     };
// // }
// //
// // impl<K: ZodType, V: ZodType> DependencyRegistration for std::collections::BTreeMap<K, V> {
// //     fn register_dependencies(cx: &mut DependencyMap)
// //     where
// //         Self: 'static,
// //     {
// //         if cx.add::<Self>() {
// //             K::register_dependencies(cx);
// //             V::register_dependencies(cx);
// //         }
// //     }
// // }

// impl<T: ZodType> ZodType for Option<T> {
// const AST: ZodExport = ZodExport {
// docs: None,
// def: ZodDefinition::Struct(Struct {
// ty: TypeDef {
// ns: Rs::NAME,
// ident: "Option",
// generics: &[Generic::new_for::<T>("T")],
// },
// fields: StructFields::Transparent {
// value: FieldValue::new_for::<T>(&phf_map! {
// 0_u64 => "T"
// }),
// optional: true,
// },
// }),
// };
// }
// impl<T: ZodType> DependencyRegistration for Option<T> {
// fn register_dependencies(cx: &mut DependencyMap)
// where
// Self: 'static,
// {
// if cx.add::<Self>() {
// T::register_dependencies(cx);
// }
// }
// }

// impl<T: ZodType, E: ZodType> ZodType for Result<T, E> {
// const AST: ZodExport =     ZodExport{docs: None, def: ZodDefinition::Literal(Literal {
// ty: TypeDef {
// ns: Rs::NAME,
// ident: "Result",
// generics: &[
// Generic::new_for::<T>("T"),
// Generic::new_for::<E>("E")
// ],

// },
// ts: "export type Result<T, E> = { Ok: T } | { Err: E };",
// zod: "export const Result = (T: z.ZodTypeAny, E: z.ZodTypeAny) => z.union([z.object({ Ok: z.lazy(() => T) }), z.object({ Err: z.lazy(() => E) })])"
// })};
// }

// impl<T: ZodType, E: ZodType> DependencyRegistration for Result<T, E> {
// fn register_dependencies(cx: &mut DependencyMap)
// where
// Self: 'static,
// {
// if cx.add::<Self>() {
// T::register_dependencies(cx);
// E::register_dependencies(cx);
// }
// }
// }

// impl_primitive!(
// std::net::Ipv4Addr,
// "Ipv4Addr",
// "string",
// "z.string().ip({ version: \"v4\" })"
// );

// impl_primitive!(
// std::net::Ipv6Addr,
// "Ipv6Addr",
// "string",
// "z.string().ip({ version: \"v6\" })"
// );

// impl_primitive!(std::net::IpAddr, "IpAddr", "string", "z.string().ip()");

// #[cfg(feature = "smol_str")]
// impl_primitive!(smol_str::SmolStr, "String", "string", "z.string()");

// #[cfg(feature = "ordered-float")]
// impl_primitive!(ordered_float::NotNan<f32>, "F32", "number", "z.number()");

// #[cfg(feature = "ordered-float")]
// impl_primitive!(ordered_float::NotNan<f64>, "F64", "number", "z.number()");

// #[cfg(test)]
// mod test {
// use std::collections::{BTreeSet, HashMap};

// use super::*;
// use pretty_assertions::assert_eq;

// #[ignore]
// #[test]
// fn option_ok() {
// assert_eq!(
// Option::<String>::AST.to_ts_string(),
// "export type Option<T> = String | undefined;"
// );
// assert_eq!(
// Option::<String>::AST.to_zod_string(),
// "export const Option = (T: z.ZodTypeAny) => z.lazy(() => T.optional());"
// );
// }

// #[test]
// fn deps_ok() {
// type T = Vec<Option<Result<Box<[HashMap<usize, bool>; 5]>, String>>>;

// assert_eq!(
// T::dependencies()
// .resolve()
// .iter()
// .map(|node| node.qualified_name())
// .collect::<BTreeSet<_>>(),
// vec![
// "Rs.Vec",
// "Rs.Option",
// "Rs.Result",
// "Rs.Array",
// "Rs.HashMap",
// "Rs.Usize",
// "Rs.Bool",
// "Rs.String",
// ]
// .into_iter()
// .map(String::from)
// .collect::<BTreeSet<_>>()
// );
// }

// #[test]
// fn tuples() {
// dbg!(<(String, usize)>::AST);
// panic!()
// }
// }
