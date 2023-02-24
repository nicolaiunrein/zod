use darling::{
    ast::{Data, Fields},
    FromDeriveInput, FromField, FromVariant,
};
use syn::{Attribute, Lit, Meta, Type};

#[derive(FromDeriveInput)]
#[darling(
    attributes(zod),
    forward_attrs(allow, doc, cfg),
    supports(struct_named, struct_newtype, enum_any)
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

pub fn get_rustdoc(attrs: &[Attribute]) -> Result<Option<String>, syn::Error> {
    let mut full_docs = String::new();
    for attr in attrs {
        match attr.parse_meta()? {
            Meta::NameValue(nv) if nv.path.is_ident("doc") => {
                if let Lit::Str(doc) = nv.lit {
                    let doc = doc.value();
                    let doc_str = doc.trim();
                    if !full_docs.is_empty() {
                        full_docs += "\n";
                    }
                    full_docs += doc_str;
                }
            }
            _ => {}
        }
    }
    Ok(if full_docs.is_empty() {
        None
    } else {
        Some(full_docs)
    })
}
