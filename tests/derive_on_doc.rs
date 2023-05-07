use paste::paste;
use pretty_assertions::assert_eq;

mod test_utils;
use test_utils::*;

const DOC: &str = "Hello World";

/// Hello World
#[derive(RequestType, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
enum SingleVariantUnit {
    A,
}

/// Hello World
#[derive(RequestType, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
enum MultiVariantUnit {
    A,
    B,
}

/// Hello World
#[derive(RequestType, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
enum SingleVariantTuple {
    A(Usize),
}

/// Hello World
#[derive(RequestType, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
enum MultiVariantTuple {
    A(Usize),
    B(Usize),
}

/// Hello World
#[derive(RequestType, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
enum SingleVariantStruct {
    A { s: String },
}

/// Hello World
#[derive(RequestType, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
enum MultiVariantStruct {
    A { s: String },
    B { num: Usize },
}

/// Hello World
#[derive(RequestType, serde::Serialize)]
#[zod(namespace = "Ns")]
struct Newtype(Usize);

/// Hello World
#[derive(RequestType, serde::Serialize)]
#[zod(namespace = "Ns")]
struct TupleStructMulti(Usize, String);

/// Hello World
#[derive(RequestType, serde::Serialize)]
#[zod(namespace = "Ns")]
struct StructSingle {
    num: Usize,
}

/// Hello World
#[derive(RequestType, serde::Serialize)]
#[zod(namespace = "Ns")]
struct StructMulti {
    num: Usize,
    s: String,
}

#[derive(zod::Namespace)]
struct Ns;

macro_rules! case {
    ($name: ident, $t: ident) => {
        paste! {
            #[test]
            fn  [<$name _schema>] () {
                assert_eq!(
                    $t::docs().map(|docs| docs.as_ref().to_string()),
                    Some(DOC.to_string())
                );
            }
        }
    };
}

case!(enum_variant_unit_single, SingleVariantUnit);
case!(enum_variant_unit_multi, MultiVariantUnit);
case!(enum_variant_tuple_single, SingleVariantTuple);
case!(enum_variant_tuple_multi, MultiVariantTuple);
case!(enum_variant_struct_single, SingleVariantStruct);
case!(enum_variant_struct_multi, MultiVariantStruct);
case!(newtype, Newtype);
case!(struct_single, StructSingle);
case!(struct_multi, StructMulti);
case!(tuple_struct_multi, TupleStructMulti);
