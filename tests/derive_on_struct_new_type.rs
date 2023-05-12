mod test_utils;
use test_utils::*;

#[test]
fn simple_ok() {
    test_case! {
        struct Test(Usize);
    }

    let json = serde_json::to_string(&Test(Usize(123))).unwrap();
    assert_eq!(json, "\"123\"");

    compare_export::<Test>(
        "export const Test = z.lazy(() => Rs.Usize);",
        "export type Test = Rs.Usize;",
    );
}

#[test]
fn generic_ok() {
    use std::collections::HashMap;
    test_case! {
        struct Test<T: RequestType>(HashMap<u8, T>);
    }

    let mut input = HashMap::<u8, u16>::new();
    input.insert(1, 2);

    let json = serde_json::to_string(&Test(input)).unwrap();
    assert_eq!(json, "{\"1\":2}");

    compare_export::<Test<u16>>(
        "export const Test = (T: z.ZodTypeAny) => z.lazy(() => Rs.HashMap(Rs.U8, Rs.U16));",
        "export type Test<T> = Rs.HashMap<Rs.U8, Rs.U16>;",
    );
}
