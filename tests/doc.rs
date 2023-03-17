use paste::paste;
use pretty_assertions::assert_eq;
use zod::{Zod, ZodType};

// /// Hello World
// #[derive(Zod, serde::Serialize)]
// #[zod(namespace = "Ns")]
// #[allow(dead_code)]
// enum SingleVariantUnit {
// A,
// }

// /// Hello World
// #[derive(Zod, serde::Serialize)]
// #[zod(namespace = "Ns")]
// #[allow(dead_code)]
// enum MultiVariantUnit {
// A,
// B,
// }

// /// Hello World
// #[derive(Zod, serde::Serialize)]
// #[zod(namespace = "Ns")]
// #[allow(dead_code)]
// enum SingleVariantTuple {
// A(usize),
// }

// /// Hello World
// #[derive(Zod, serde::Serialize)]
// #[zod(namespace = "Ns")]
// #[allow(dead_code)]
// enum MultiVariantTuple {
// A(usize),
// B(usize),
// }

// /// Hello World
// #[derive(Zod, serde::Serialize)]
// #[zod(namespace = "Ns")]
// #[allow(dead_code)]
// enum SingleVariantStruct {
// A { s: String },
// }

// /// Hello World
// #[derive(Zod, serde::Serialize)]
// #[zod(namespace = "Ns")]
// #[allow(dead_code)]
// enum MultiVariantStruct {
// A { s: String },
// B { num: usize },
// }

/// Hello World
#[derive(Zod, serde::Serialize)]
#[zod(namespace = "Ns")]
struct Newtype(usize);

// TODO
// /// Hello World
// #[derive(Zod, serde::Serialize)]
// #[zod(namespace = "Ns")]
// struct TupleStructMulti(usize, String);

/// Hello World
#[derive(Zod, serde::Serialize)]
#[zod(namespace = "Ns")]
struct StructSingle {
    num: usize,
}

/// Hello World
#[derive(Zod, serde::Serialize)]
#[zod(namespace = "Ns")]
struct StructMulti {
    num: usize,
    s: String,
}

#[derive(zod::Namespace)]
struct Ns;

fn main() {}

const DOC: &str = "Hello World";

macro_rules! case {
    ($name: ident, $t: ident) => {
        paste! {
            #[test]
            fn  [<$name _schema>] () {
                assert_eq!(
                    $t::AST.docs(),
                    Some(DOC)
                );
            }
        }
    };
}

// case!(enum_variant_unit_single, SingleVariantUnit);
// case!(enum_variant_unit_multi, MultiVariantUnit);
// case!(enum_variant_tuple_single, SingleVariantTuple);
// case!(enum_variant_tuple_multi, MultiVariantTuple);
// case!(enum_variant_struct_single, SingleVariantStruct);
// case!(enum_variant_struct_multi, MultiVariantStruct);

case!(newtype, Newtype);
case!(struct_single, StructSingle);
case!(struct_multi, StructMulti);
// TODO
// case!(tuple_struct_multi, TupleStructMulti, "z.number", "number");
