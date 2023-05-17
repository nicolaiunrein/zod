mod test_utils;
use pretty_assertions::assert_eq;
use test_utils::*;

#[test]
fn generic_structs() {
    test_case! {
        #[derive(serde::Deserialize, Debug)]
        pub struct Generic<T1: zod::RequestType, T2: zod::RequestType> {
            value: String,
            t: T1,
            v: T2,
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug, RequestType)]
        #[zod(namespace = "Ns")]
        pub struct GenericTuple<T1: zod::RequestType, T2: zod::RequestType>(
            String,
            T1,
            T2,
        );

        #[derive(serde::Serialize, serde::Deserialize, Debug, RequestType)]
        #[zod(namespace = "Ns")]
        pub struct User {
            strukt: Generic<String, Usize>,
            tuple: GenericTuple<String, Usize>,
        }
    }

    compare_export::<User>(
        "export const User = z.lazy(() => z.object({strukt: Ns.Generic(Rs.String, Rs.Usize), tuple: Ns.GenericTuple(Rs.String, Rs.Usize)}));",
        "export interface User { strukt: Ns.Generic<Rs.String, Rs.Usize>, tuple: Ns.GenericTuple<Rs.String, Rs.Usize> }",
    );

    compare_export::<Generic<(), ()>>(
        "export const Generic = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.object({value: Rs.String, t: T1, v: T2});",
        "export interface Generic<T1, T2> { value: Rs.String, t: T1, v: T2 }",
    );

    compare_export::<GenericTuple<(), ()>>(
        "export const GenericTuple = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.tuple([Rs.String, T1, T2]);",
        "export type GenericTuple<T1, T2> = [ Rs.String, T1, T2 ];",
    );
}

#[ignore]
#[test]
fn generic_enums() {
    test_case! {
        #[derive(serde::Deserialize, Debug)]
        pub enum Generic<T1: zod::RequestType, T2: zod::RequestType> {
            T1{
                inner: T1
            },
            T2 {
                inner: T2
            },
            String {
                inner: String
            }
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug, RequestType)]
        #[zod(namespace = "Ns")]
        pub enum GenericTuple<T1: zod::RequestType, T2: zod::RequestType> {
            T1(T1),
            T2(T2),
            String(String),
        }

    }

    compare_export::<Generic<(), ()>>(
        "export const Generic = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) = > z.tuple([z.object({T1: z.object({inner: T1})}), z.object({T2: z.object({inner: T2})}), z.object({String: z.object({inner: Rs.String})})])",
        "Todo",
        // "export const Generic = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.tuple({value: Rs.String, t: T1, v: T2});",
        // "export interface Generic<T1, T2> { value: Rs.String, t: T1, v: T2 }",
    );

    compare_export::<GenericTuple<(), ()>>(
        "export const GenericTuple = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.tuple([Rs.String, T1, T2]);",
        "export type GenericTuple<T1, T2> = [ Rs.String, T1, T2 ];",
    );
}

// #[test]
// fn flipped_args() {
//     test_case! {
//         #[derive(serde::Deserialize, Debug)]
//         pub struct Generic<T1: zod::RequestType, T2: zod::RequestType> {
//             t1: T1,
//             t2: T2,
//         }
//
//         type Flipped<T1, T2> = Generic<T2, T1>;
//
//         #[derive(serde::Serialize, serde::Deserialize, Debug, RequestType)]
//         #[zod(namespace = "Ns")]
//          struct MyType<T: zod::RequestType> {
//              ok: Generic<String, T>,
//              flipped: Flipped<T, String> // <-- equals MyGeneric<String, T>
//          }
//     }
//
//     let schema = MyType::<()>::export().schema;
//     match schema {
//         zod::core::ast::ExportSchema::Object(obj) => {
//             let mut fields = obj.fields().iter();
//             let first = fields.next().unwrap();
//             let second = fields.next().unwrap();
//
//             assert_eq!(first.value(), second.value());
//         }
//         _ => panic!("unexpected schema"),
//     }
// }

// #[ignore]
// #[test]
// fn nested_generics() {
//     test_case! {
//         #[derive(serde::Deserialize, Debug)]
//         pub struct Generic<T: zod::RequestType> {
//             t: T,
//         }
//
//         #[derive(serde::Serialize, serde::Deserialize, Debug, RequestType)]
//         #[zod(namespace = "Ns")]
//         pub struct User<T: zod::RequestType> {
//             value: Generic<Option<T>>,
//         }
//
//
//     }
//
//     // TODO
//     compare_export::<User<String>>(
//         "Todo",
//         "Todo",
//         // "export const User = z.lazy(() => z.object({value: Ns.Generic(Rs.Option(Rs.String)) }));",
//         // "export interface User { value: Ns.Generic<Rs.Option<Rs.String>> }",
//     );
// }
//
// #[ignore]
// #[test]
// fn generic_ok() {
//     test_case! {
//         #[derive(serde::Deserialize, Debug)]
//         struct Generic<T1: RequestType, T2: RequestType>(T1);
//
//
//         #[derive(serde::Serialize, serde::Deserialize, Debug, RequestType)]
//         #[zod(namespace = "Ns")]
//         struct Test<T1: RequestType, T2: RequestType>(T1, Generic<u8, T2>);
//     }
//
//     println!("{:#?}", Test::<u8, u16>::export());
//     compare_export::<Test<u8, u16>>(
//         "export const Test = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.lazy(() => z.tuple([T1, Ns.Generic(Rs.U8, T2)]);",
//         "TODO"
//         // "export type Test<T> = Rs.HashMap<Rs.U8, Rs.U16>;",
//     );
// }
