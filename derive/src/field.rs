use crate::config::FieldConfig;
use crate::error::Error;
use darling::ToTokens;
use quote::quote;
use serde_derive_internals::ast::Field as SerdeField;
use syn::Type;

use crate::utils::get_zod;

#[derive(Clone)]
pub struct Field {
    pub ty: Type,
    pub config: FieldConfig,
}

impl Field {
    pub fn new<'a>(value: &'a SerdeField) -> Result<Self, Error> {
        Ok(Self {
            ty: value.original.ty.clone(),
            config: FieldConfig::new(&value.attrs)?,
        })
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = &self.ty;

        let optional = if self.config.default {
            quote!(.optional())
        } else {
            quote!()
        };

        let zod = get_zod();

        match self.config.name {
            Some(ref name) => tokens.extend(quote! {
                #zod::core::ast::NamedField::new::<#ty>(#name) #optional
            }),
            None => tokens.extend(quote! {
                #zod::core::ast::TupleField::new::<#ty>() #optional
            }),
        }
    }
}

#[derive(Clone)]
pub struct FilteredFields(Vec<Field>);

impl FilteredFields {
    pub fn new(inner: Vec<Field>) -> Self {
        let inner = inner.into_iter().filter(|f| !f.config.ignored).collect();
        Self(inner)
    }
    pub fn iter(&self) -> impl Iterator<Item = &Field> {
        self.0.iter()
    }
}

impl ToTokens for FilteredFields {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let inner = &self.0;
        tokens.extend(quote!(#(#inner),*));
    }
}
