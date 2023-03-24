use darling::{FromField, ToTokens};
use quote::quote;
use syn::{Attribute, Type};

use crate::utils::get_zod;

#[derive(FromField)]
#[darling(attributes(zod), forward_attrs(allow, doc, cfg, serde))]
pub struct Field {
    pub ident: Option<syn::Ident>,
    pub ty: Type,
    pub attrs: Vec<Attribute>,
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = &self.ty;
        let optional = quote!();
        let zod = get_zod();

        match self.ident {
            Some(ref name) => tokens.extend(quote! {
                #zod::core::ast::NamedField::new::<#ty>(#name) #optional
            }),
            None => tokens.extend(quote! {
                #zod::core::ast::TupleField::new::<#ty>() #optional
            }),
        }
    }
}
