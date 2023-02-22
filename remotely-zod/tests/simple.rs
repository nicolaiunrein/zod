use remotely_zod::{zod, Codegen};

#[derive(zod)]
#[zod(ns = "abc")]
struct Z {
    b: bool,
}

#[derive(zod)]
#[zod(ns = "abc")]
struct X {
    num: i32,
    s: String,
    z: Z,
}

#[test]
fn is_ok() {
    assert_eq!(
        X::code(),
        format!(
            "z.object({{num: {},\ns: {},\nz: abc.Z,}})",
            i32::code(),
            String::code()
        )
    );
}
