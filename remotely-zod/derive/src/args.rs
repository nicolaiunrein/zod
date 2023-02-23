use darling::{ast::Data, FromDeriveInput, FromField, FromVariant};
use syn::Type;

#[derive(FromDeriveInput)]
#[darling(
    attributes(zod),
    forward_attrs(allow, doc, cfg),
    supports(struct_named, struct_newtype, enum_named)
)]
pub struct Input {
    pub ident: syn::Ident,
    pub data: Data<EnumVariant, StructField>,
    pub namespace: syn::Path,
}

#[derive(FromVariant, Clone)]
pub struct EnumVariant {}

#[derive(FromField, Clone)]
pub struct StructField {
    pub ident: Option<syn::Ident>,
    pub ty: Type,
}
