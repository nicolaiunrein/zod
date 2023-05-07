use pretty_assertions::assert_eq;
use zod::core::RequestTypeVisitor;
use zod::Namespace;
use zod::RequestType;
use zod::ResponseType;
use zod_core::ResponseTypeVisitor;

#[derive(RequestType, serde::Deserialize, ResponseType, serde::Serialize, Clone)]
#[zod(namespace = "Ns")]
#[serde(from = "String", into = "zod::types::Usize")]
pub struct Inner {
    value: String,
}

#[derive(RequestType, serde::Deserialize, ResponseType, serde::Serialize, Clone)]
#[zod(namespace = "Ns")]
#[serde(from = "Inner", into = "Inner")]
pub struct Compound {
    inner: Inner,
}

impl From<String> for Inner {
    fn from(value: String) -> Self {
        Self { value }
    }
}

impl From<Inner> for zod::types::Usize {
    fn from(outer: Inner) -> Self {
        outer.value.len().into()
    }
}

impl From<Compound> for Inner {
    fn from(value: Compound) -> Self {
        value.inner
    }
}

impl From<Inner> for Compound {
    fn from(inner: Inner) -> Self {
        Compound { inner }
    }
}

#[derive(Namespace)]
struct Ns;

#[test]
fn request_ok() {
    assert_eq!(
        <String as RequestType>::export().schema,
        <Inner as RequestType>::export().schema
    );
}

#[test]
fn response_ok() {
    assert_eq!(
        <zod::types::Usize as ResponseType>::export().schema,
        <Inner as ResponseType>::export().schema
    );
}

#[test]
fn registers_dependencies() {
    let req_deps = <Inner as RequestTypeVisitor>::dependencies();
    let req_exports = req_deps.values().collect::<Vec<_>>();

    let res_deps = <Inner as ResponseTypeVisitor>::dependencies();
    let res_exports = res_deps.values().collect::<Vec<_>>();

    let string_deps = <String as RequestTypeVisitor>::dependencies();
    let string_exports = string_deps.values().collect::<Vec<_>>();

    let usize_deps = <zod::types::Usize as RequestTypeVisitor>::dependencies();
    let usize_exports = usize_deps.values().collect::<Vec<_>>();

    assert_eq!(req_exports, string_exports);
    assert_eq!(res_exports, usize_exports);
}

#[test]
fn registers_compound_dependencies() {
    let req_deps = <Compound as RequestTypeVisitor>::dependencies();
    let req_exports = req_deps.values().collect::<Vec<_>>();

    let res_deps = <Compound as ResponseTypeVisitor>::dependencies();
    let res_exports = res_deps.values().collect::<Vec<_>>();

    let string_deps = <String as RequestTypeVisitor>::dependencies();
    let string_exports = string_deps.values().collect::<Vec<_>>();

    let usize_deps = <zod::types::Usize as RequestTypeVisitor>::dependencies();
    let usize_exports = usize_deps.values().collect::<Vec<_>>();

    assert_eq!(req_exports, string_exports);
    assert_eq!(res_exports, usize_exports);
}
