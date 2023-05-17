use crate::error::Error;
use crate::zod_type::config::FieldConfig;
use darling::ToTokens;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

use crate::utils::get_zod;

use super::Derive;

#[derive(Clone, Debug)]
pub(crate) struct Field {
    pub ty: Type,
    pub config: FieldConfig,
    pub generic: Option<Ident>,
    generic_idents: Vec<Ident>,
}

fn get_generic(ty: &Type, generics: &[Ident]) -> Option<Ident> {
    match ty {
        Type::Path(p) => match p.path.get_ident() {
            Some(ident) => generics
                .iter()
                .find(|gen| gen == &ident)
                .map(|i| Ident::clone(i)),

            None => None,
        },

        _ => None,
    }
}

fn reference(ty: &Type, generic_idents: &[Ident], derive: Derive) -> TokenStream {
    let zod = get_zod();

    match ty {
        Type::Path(p) => {
            if let Some(i) = p.path.get_ident() {
                if let Some(index) = generic_idents.iter().position(|ii| ii == i) {
                    let index = syn::Index::from(index);

                    return quote!(#zod::core::Generic<#index>);
                }
            }
            for seg in p.path.segments.iter() {
                match &seg.arguments {
                    syn::PathArguments::AngleBracketed(inner) => {
                        let args = inner.args.iter().map(|arg| match arg {
                            syn::GenericArgument::Type(t) => reference(t, generic_idents, derive),
                            _ => todo!(),
                        });

                        let ident = &seg.ident;
                        return quote!(#ident<#(#args),*>);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    quote!(#ty)
}

impl Field {
    pub(crate) fn new(
        ty: &Type,
        config: FieldConfig,
        generic_idents: Vec<Ident>,
    ) -> Result<Self, Error> {
        let generic = get_generic(ty, &generic_idents);

        // if !nested_generics.is_empty() {
        //     todo!("nested generics")
        // }

        Ok(Self {
            ty: ty.clone(),
            config,
            generic,
            generic_idents,
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

        let reference = reference(ty, &self.generic_idents, self.config.derive);
        // let reference = if let Some(ident) = position(ty, &self.generic_idents) {
        //     let s = ident.to_string();
        //     quote!(
        //         #zod::core::ast::Ref:: generic(#s)
        //     )
        // } else {
        //     quote!(
        //         #zod::core::ast::Ref:: #req_res ::<#ty>()
        //     )
        // };
        //
        let req_res = match self.config.derive {
            Derive::Request => quote!(new_req),
            Derive::Response => quote!(new_res),
        };

        match (&self.generic, &self.config.name) {
            (None, Some(ref name)) => tokens.extend(quote! {
                #zod::core::ast::NamedField::new(#name, #zod::core::ast::Ref::#req_res::<#reference>()) #optional
            }),
            (None, None) => tokens.extend(quote! {
                #zod::core::ast::TupleField:: new(#zod::core::ast::Ref::#req_res::<#reference>()) #optional
            }),

            (Some(ident), Some(ref name)) => {
                let value = ident.to_string();

                tokens.extend(quote! {
                    #zod::core::ast::NamedField::new(#name, #zod::core::ast::Ref::generic(#value)) #optional
                })
            }
            (Some(ident), None) => {
                let value = ident.to_string();

                tokens.extend(quote! {
                    #zod::core::ast::TupleField::new(#zod::core::ast::Ref::generic(#value)) #optional
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
        generics: Vec<Ident>,
    ) -> Result<Self, Error> {
        let inner = fields
            .into_iter()
            .map(|(ty, config)| Field::new(ty, config, generics.clone()))
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
