use darling::ToTokens;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Lit, Meta};

pub struct RustDocs {
    inner: Option<String>,
}

impl RustDocs {
    pub fn from_attrs(attrs: &[syn::Attribute]) -> Result<Self, syn::Error> {
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
            Self { inner: None }
        } else {
            Self {
                inner: Some(full_docs),
            }
        })
    }
}

impl ToTokens for RustDocs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self.inner {
            Some(ref docs) => {
                let docs = format!(
                    "/**\n{}*/\n",
                    docs.lines()
                        .map(|line| format!("* {}\n", line))
                        .collect::<String>()
                );
                tokens.extend(quote!(#docs))
            }
            None => tokens.extend(quote!("")),
        }
    }
}
