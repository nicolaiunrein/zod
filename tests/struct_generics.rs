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
        pub struct User {
            value: Generic<String, Usize>,
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug, RequestType)]
        #[zod(namespace = "Ns")]
        pub struct UserMixed<New: zod::RequestType> {
            value: Generic<New, Usize>,
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug, RequestType)]
        #[zod(namespace = "Ns")]
        pub struct UserTransparent<T2: zod::RequestType, T1: zod::RequestType> {
            value: Generic<T1, T2>,
        }

    }

    compare_export::<User>(
        "export const User = z.lazy(() => z.object({value: Ns.Generic(Rs.String, Rs.Usize)}));",
        "export interface User { value: Ns.Generic<Rs.String, Rs.Usize> }",
    );

    compare_export::<Generic<(), ()>>(
        "export const Generic = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.object({value: Rs.String, t: T1, v: T2});",
        "export interface Generic<T1, T2> { value: Rs.String, t: T1, v: T2 }",
    );
}

#[test]
fn flipped_argus() {
    test_case! {
        #[derive(serde::Deserialize, Debug)]
        pub struct Generic<T1: zod::RequestType, T2: zod::RequestType> {
            t1: T1,
            t2: T2,
        }

        type Flipped<T1, T2> = Generic<T2, T1>;

        #[derive(serde::Serialize, serde::Deserialize, Debug, RequestType)]
        #[zod(namespace = "Ns")]
         struct MyType<T: zod::RequestType> {
             ok: Generic<String, T>,
             flipped: Flipped<T, String> // <-- equals MyGeneric<String, T>
         }
    }

    let schema = MyType::<()>::export().schema;
    match schema {
        zod::ast::ExportSchema::Object(obj) => {
            let mut fields = obj.fields().iter();
            let first = fields.next().unwrap();
            let second = fields.next().unwrap();

            assert_eq!(first.value(), second.value());
        }
        _ => panic!("unexpected schema"),
    }
}
