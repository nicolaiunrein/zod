use darling::FromDeriveInput;

#[derive(FromDeriveInput)]
#[darling(
    attributes(zod),
    forward_attrs(allow, doc, cfg),
    supports(struct_named, struct_newtype, struct_tuple, enum_any)
)]

pub(super) struct ZodAttrs {
    pub namespace: syn::Path,
    pub custom_suffix: Option<String>,
}
