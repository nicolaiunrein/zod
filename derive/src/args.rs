use darling::{
    ast::{Data, Fields},
    FromDeriveInput, FromField, FromVariant,
};
use syn::{Attribute, Type};

#[derive(FromDeriveInput)]
#[darling(
    attributes(zod),
    forward_attrs(allow, doc, cfg),
    supports(struct_named, struct_newtype, struct_tuple, enum_any)
)]
pub struct Input {
    pub ident: syn::Ident,
    pub data: Data<EnumVariant, StructField>,
    pub namespace: syn::Path,
    pub attrs: Vec<Attribute>,
}

#[derive(FromVariant, Clone)]
pub struct EnumVariant {
    pub ident: syn::Ident,
    pub fields: Fields<EnumField>,
}

#[derive(FromField, Clone)]
pub struct StructField {
    pub ident: Option<syn::Ident>,
    pub ty: Type,
}

#[derive(FromField, Clone)]
pub struct EnumField {
    pub ident: Option<syn::Ident>,
    pub ty: Type,
}

#[derive(FromDeriveInput)]
#[darling(attributes(namespace), forward_attrs(allow, doc, cfg))]
pub struct NamespaceInput {
    pub vis: syn::Visibility,
    pub ident: syn::Ident,
    pub data: Data<EnumVariant, StructField>,
    pub name: Option<String>,
    pub attrs: Vec<Attribute>,
    pub generics: syn::Generics,
}
