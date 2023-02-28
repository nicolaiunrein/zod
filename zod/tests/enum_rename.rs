use pretty_assertions::assert_eq;
use zod::{zod, Codegen, Namespace};

mod test_utils;

#[test]
fn serde_name_struct() {
    test_case! {
        #[serde(rename= "Hello")]
        enum Test {
            HelloWorld { s: String },
            AnotherValue { num: usize },
        }
    }

    assert_eq!(Test::type_name(), "Ns.Hello");
}

#[test]
fn serde_name_tuple() {
    test_case! {
    #[serde(rename= "Hello")]
    enum Test {
        HelloWorld(String, usize),
        AnotherValue(usize, usize)
        }
    }
    assert_eq!(Test::type_name(), "Ns.Hello");
}

#[test]
fn serde_name_unit() {
    test_case! {
        #[serde(rename= "Hello")]
        enum Test {
            HelloWorld,
            AnotherValue
        }
    }
    assert_eq!(Test::type_name(), "Ns.Hello");
}
