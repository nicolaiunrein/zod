use darling::FromField;
use syn::Type;

#[derive(FromField, Clone)]
pub struct Field {
    pub ident: Option<syn::Ident>,
    pub ty: Type,
}
