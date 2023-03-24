use crate::docs::RustDocs;
use darling::FromField;
use syn::Type;

#[derive(FromField)]
#[darling(attributes(zod), forward_attrs(allow, doc, cfg, serde))]
pub struct Field {
    pub ident: Option<syn::Ident>,
    pub ty: Type,
    pub doc: Option<String>,
}
