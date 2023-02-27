use paste::paste;
use pretty_assertions::assert_eq;
use remotely::zod;
use remotely_core::codegen::namespace::Namespace;
use remotely_zod::Codegen;

/// Hello World
#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
enum SingleVariantUnit {
    A,
}

/// Hello World
#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
enum MultiVariantUnit {
    A,
    B,
}

/// Hello World
#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
enum SingleVariantTuple {
    A(usize),
}

/// Hello World
#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
enum MultiVariantTuple {
    A(usize),
    B(usize),
}

/// Hello World
#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
enum SingleVariantStruct {
    A { s: String },
}

/// Hello World
#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
#[allow(dead_code)]
enum MultiVariantStruct {
    A { s: String },
    B { num: usize },
}

/// Hello World
#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
struct Newtype(usize);

// TODO
// /// Hello World
// #[derive(zod, serde::Serialize)]
// #[zod(namespace = "Ns")]
// struct TupleStructMulti(usize, String);

/// Hello World
#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
struct StructSingle {
    num: usize,
}

/// Hello World
#[derive(zod, serde::Serialize)]
#[zod(namespace = "Ns")]
struct StructMulti {
    num: usize,
    s: String,
}

struct Ns {}

impl Namespace for Ns {
    const NAME: &'static str = "Ns";
    type Req = NsReq;
}

#[derive(serde::Deserialize)]
struct NsReq;

fn main() {}

const DOC: &str = "/**\n* Hello World\n*/\n";

macro_rules! case {
    ($name: ident, $t: ident) => {
        paste! {
            #[test]
            fn  [<$name _schema>] () {
                assert_eq!(
                    $t::docs(),
                    Some(DOC)
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
// TODO
// case!(tuple_struct_multi, TupleStructMulti, "z.number", "number");
