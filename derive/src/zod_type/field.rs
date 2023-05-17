use crate::error::Error;
use crate::zod_type::config::FieldConfig;
use darling::ToTokens;
use quote::quote;
use syn::{Ident, Type};

use crate::utils::get_zod;

use super::Derive;

#[derive(Clone, Debug)]
pub(crate) struct Field {
    pub ty: Type,
    pub config: FieldConfig,
    pub generic: Option<Ident>,
    pub nested_generics: Vec<Ident>,
}

fn get_generic(ty: &Type, generics: &[&Ident]) -> Option<Ident> {
    match ty {
        Type::Path(p) => match p.path.get_ident() {
            Some(ident) => generics
                .iter()
                .find(|gen| gen == &&ident)
                .map(|i| Ident::clone(i)),

            None => None,
        },

        _ => None,
    }
}

fn get_nested_generics(ty: &Type, generics: &[&Ident]) -> Vec<Ident> {
    match ty {
        Type::Path(p) => match p.path.get_ident() {
            Some(_) => Vec::new(),
            None => get_nested_generics_inner(ty, generics),
        },
        _ => Vec::new(),
    }
}

fn get_nested_generics_inner(ty: &Type, generics: &[&Ident]) -> Vec<Ident> {
    match ty {
        Type::Path(p) => match p.path.get_ident() {
            Some(ident) => generics
                .iter()
                .filter(|gen| gen == &&ident)
                .map(|i| Ident::clone(i))
                .collect(),

            None => p
                .path
                .segments
                .iter()
                .map(|seg| match seg.arguments {
                    syn::PathArguments::None => Vec::new(),
                    syn::PathArguments::AngleBracketed(ref inner) => inner
                        .args
                        .iter()
                        .map(|arg| match arg {
                            syn::GenericArgument::Type(t) => get_nested_generics_inner(t, generics),
                            _ => Vec::new(),
                        })
                        .collect(),
                    syn::PathArguments::Parenthesized(_) => Vec::new(),
                })
                .map(|inner| inner.into_iter().map(|i| i.into_iter()).flatten())
                .flatten()
                .collect(),
        },

        _ => Vec::new(),
    }
}

impl Field {
    pub(crate) fn new(ty: &Type, config: FieldConfig, generics: &[&Ident]) -> Result<Self, Error> {
        let generic = get_generic(ty, generics);
        let nested_generics = get_nested_generics(ty, generics);
        if !nested_generics.is_empty() {
            todo!("nested generics")
        }

        Ok(Self {
            ty: ty.clone(),
            config,
            generic,
            nested_generics,
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
                let name = ident.to_string();

                tokens.extend(quote! {
                    #zod::core::ast::TupleField::generic(#name) #optional
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

        let fields: Vec<_> = inner.into_iter().filter(|f| !f.config.ignored).collect();

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
