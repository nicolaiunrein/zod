#![allow(dead_code)]

pub const A: &str = "z.literal(\"A\")";
pub const B: &str = "z.literal(\"B\")";
pub const NULL: &str = "z.null()";

pub fn discriminated_union(t: impl AsRef<str>, items: &[impl AsRef<str>]) -> String {
    format!(
        "z.discriminatedUnion(\"{}\", [{}])",
        t.as_ref(),
        items
            .iter()
            .map(|i| i.as_ref())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn zod_union(items: &[impl AsRef<str>]) -> String {
    format!(
        "z.union([{}])",
        items
            .iter()
            .map(|i| i.as_ref())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[macro_export]
macro_rules! object {
    ($($k: tt: $v:expr),*) => {
        zod_obj(&[$((stringify!($k), $v)),*])
    };
}

pub fn zod_obj(fields: &[(impl AsRef<str>, impl AsRef<str>)]) -> String {
    let inner = fields
        .iter()
        .map(|(k, v)| format!("{}: {}", k.as_ref(), v.as_ref()))
        .collect::<Vec<_>>();

    format!("z.object({{ {} }})", inner.join(", "))
}

pub fn tuple(fields: &[impl AsRef<str>]) -> String {
    let inner = fields.iter().map(|f| f.as_ref()).collect::<Vec<_>>();
    format!("z.tuple([{}])", inner.join(", "))
}

pub fn adj_tagged(variant: &str, inner: impl AsRef<str>) -> String {
    zod_obj(&[
        ("type", format!("z.literal(\"{variant}\")")),
        ("content", inner.as_ref().to_string()),
    ])
}

pub fn optional(inner: impl AsRef<str>) -> String {
    format!("{}.optional()", inner.as_ref())
}

#[macro_export]
macro_rules! test_case {
    ($($decl: tt)+) => {
        #[derive(zod::Zod, serde::Serialize)]
        #[zod(namespace = "Ns")]
        #[allow(dead_code)]
        $($decl)+

        struct Ns {}

        impl Namespace for Ns {
            const NAME: &'static str = "Ns";
        }
    };
}
