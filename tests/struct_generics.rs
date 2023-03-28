mod test_utils;
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

    // compare_export::<Generic<(), ()>>(
    // "export const Generic = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.lazy(() => z.object({value: Rs.String, t: T1, v: T2}));",
    // "export interface Generic<T1, T2> { value: Rs.String, t: T1, v: T2 }",
    // );

    let ex = <Generic<(), ()> as RequestType>::AST;
    println!("{ex:#?}");
    panic!()

    // compare(
    // ex.export().unwrap().to_ts_string(),
    // "export interface Generic<T1, T2> { value: Rs.String, t: T1, v: T2 }",
    // );
}
