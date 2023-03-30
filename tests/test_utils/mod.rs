#![allow(dead_code)]
use pretty_assertions::assert_eq;
pub use unindent::Unindent;
pub use zod::ast::Compiler;
pub use zod::types::Usize;
pub use zod::RequestType;

pub fn compare(input: impl AsRef<str>, expected: &str) {
    assert_eq!(normalize(input.as_ref()), normalize(expected))
}

pub fn compare_export<T: zod::RequestType>(expected_zod: &str, expected_ts: &str) {
    let export = <T as zod::RequestType>::EXPORT;

    compare(&export.to_zod_string(), expected_zod);
    compare(&export.to_ts_string(), expected_ts);
}

fn expand(input: String) -> String {
    let mut out = String::new();

    let mut iter = input.chars().peekable();

    while let Some(current) = iter.next() {
        if [',', '{'].contains(&current) {
            out.push(current);
            out.push(' ');
            continue;
        }

        if ['}'].contains(&current) {
            out.push(' ');
            out.push(current);
            continue;
        }

        if current == '=' && iter.peek().is_some() && iter.peek() != Some(&'>') {
            out.push(' ');
            out.push(current);
            out.push(' ');
            continue;
        }

        if current == '=' && iter.peek() == Some(&'>') {
            out.push(' ');
            out.push(current);
            out.push('>');
            out.push(' ');
            iter.next();
            continue;
        }

        out.push(current)
    }

    out
}

#[cfg(test)]
fn normalize(input: impl AsRef<str>) -> String {
    let mut out = String::new();
    let mut iter = input.as_ref().chars().peekable();

    let mut last: Option<char> = None;

    while let Some(current) = iter.next() {
        if let Some(next) = iter.peek() {
            if current.is_whitespace() && next.is_whitespace() {
                continue;
            }

            if current.is_whitespace() && !next.is_alphanumeric() {
                continue;
            }

            if let Some(last) = last {
                if current.is_whitespace() && !last.is_alphanumeric() {
                    continue;
                }
            }
        }

        out.push(current);
        last = Some(current)
    }
    expand(out)
}

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

    #[derive(zod::Namespace)]
    struct Ns;

    #[derive(zod::RequestType, serde::Serialize)]
    #[zod(namespace = "Ns")]
    #[allow(dead_code)]
    $($decl)+

    };
}
