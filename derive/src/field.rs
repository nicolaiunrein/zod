use crate::config::{Derive, FieldConfig};
use crate::error::Error;
use darling::ToTokens;
use quote::quote;
use syn::{Ident, Type};

use crate::utils::get_zod;

#[derive(Clone, Debug)]
pub(crate) struct Field {
    pub ty: Type,
    pub config: FieldConfig,
    pub generic: Option<Ident>,
}

impl Field {
    pub(crate) fn new(ty: &Type, config: FieldConfig, generics: &[&Ident]) -> Result<Self, Error> {
        let generic = match ty {
            Type::Path(p) => p
                .path
                .get_ident()
                .map(|ident| generics.iter().find(|gen| gen == &&ident))
                .flatten()
                .map(|i| Ident::clone(i)),

            _ => None,
        };

        Ok(Self {
            ty: ty.clone(),
            config,
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

        let req_res = match self.config.derive {
            Derive::Request => quote!(new_req),
            Derive::Response => quote!(new_res),
        };

        match (&self.generic, &self.config.name) {
            (None, Some(ref name)) => tokens.extend(quote! {
                #zod::core::ast::NamedField::new_req::<#ty>(#name) #optional
            }),
            (None, None) => tokens.extend(quote! {
                #zod::core::ast::TupleField:: #req_res ::<#ty>() #optional
            }),

            (Some(ident), Some(ref name)) => {
                let value = ident.to_string();

                tokens.extend(quote! {
                    #zod::core::ast::NamedField::generic(#name, #value) #optional
                })
            }
            (Some(ident), None) => {
                let _value = ident.to_string();

                tokens.extend(quote! {
                    #zod::core::ast::TupleField:: #req_res  ::<#ty>() #optional
                })
            }
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct FilteredFields {
    fields: Vec<Field>,
}

impl FilteredFields {
    pub(crate) fn new(
        fields: Vec<(&Type, FieldConfig)>,
        generics: &[&Ident],
    ) -> Result<Self, Error> {
        let inner = fields
            .into_iter()
            .map(|(ty, config)| Field::new(ty, config, generics))
            .collect::<Result<Vec<_>, _>>()?;

        let fields = inner.into_iter().filter(|f| !f.config.ignored).collect();

        Ok(Self { fields })
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter()
    }
}

impl ToTokens for FilteredFields {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let inner = &self.fields;
        tokens.extend(quote!(#(#inner),*));
    }
}
