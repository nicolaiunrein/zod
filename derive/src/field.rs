use crate::config::FieldConfig;
use crate::error::Error;
use crate::node::Derive;
use darling::ToTokens;
use quote::quote;
use serde_derive_internals::ast::Field as SerdeField;
use syn::{Ident, Type};

use crate::utils::get_zod;

#[derive(Clone, Debug)]
pub struct Field {
    pub ty: Type,
    pub config: FieldConfig,
    pub generic: Option<Ident>,
}

impl Field {
    pub fn new<'a>(
        value: &'a SerdeField,
        derive: Derive,
        generic: Option<Ident>,
    ) -> Result<Self, Error> {
        Ok(Self {
            ty: value.original.ty.clone(),
            config: FieldConfig::new(&value.attrs, derive)?,
            generic,
        })
    }
}

impl ToTokens for Field {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ty = &self.ty;

        let optional = if self.config.required {
            quote!()
        } else {
            quote!(.optional())
        };

        let zod = get_zod();

        match (&self.generic, &self.config.name) {
            (None, Some(ref name)) => tokens.extend(quote! {
                #zod::core::ast::NamedField::new::<#ty>(#name) #optional
            }),
            (None, None) => tokens.extend(quote! {
                #zod::core::ast::TupleField::new::<#ty>() #optional
            }),

            (Some(ident), Some(ref name)) => {
                let value = ident.to_string();

                tokens.extend(quote! {
                    #zod::core::ast::NamedField::generic(#name, #value) #optional
                })
            }
            (Some(ident), None) => {
                let value = ident.to_string();

                tokens.extend(quote! {
                    // todo
                    #zod::core::ast::TupleField::new::<#ty>() #optional
                })
            }
        }
    }
}

#[derive(Clone, Debug)]
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
