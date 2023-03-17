use zod::ast::{FormatTypescript, FormatZod};
use zod::ZodType;

mod test_utils;
use test_utils::*;

#[test]
fn generic_structs() {
    test_case! {
        #[derive(serde::Deserialize, Debug)]
        pub struct Generic<T1, T2> {
            value: String,
            t: T1,
            v: T2,
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug, zod::Zod)]
        #[zod(namespace = "Ns")]
        pub struct User {
            value: Generic<String, usize>,
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug, zod::Zod)]
        #[zod(namespace = "Ns")]
        pub struct UserMixed<New> {
            value: Generic<New, usize>,
        }

        #[derive(serde::Serialize, serde::Deserialize, Debug)]
        // #[zod(namespace = "Ns")]
        pub struct UserTransparent<T2, T1> {
            value: Generic<T1, T2>,
        }
    }

    compare(
        Generic::<(), ()>::AST.to_zod_string(),
        "export const Generic = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.lazy(() => z.object({value: Rs.String, t: T1, v: T2}));",
    );

    compare(
        Generic::<(), ()>::AST.to_ts_string(),
        "export interface Generic<T1, T2> { value: Rs.String, t: T1, v: T2 }",
    );

    compare(
        User::AST.to_zod_string(),
        "export const User = z.lazy(() => z.object({value: Ns.Generic(Rs.String, Rs.Usize)}));",
    );

    compare(
        User::AST.to_ts_string(),
        "export interface User { value: Ns.Generic<Rs.String, Rs.Usize> }",
    );
}
