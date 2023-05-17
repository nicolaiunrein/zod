use crate::error::Error;
use crate::utils::get_zod;
use crate::zod_type::field::{Field, FilteredFields};
use crate::zod_type::Derive;
use darling::ToTokens;
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

pub(super) enum Schema<'a> {
    Object(ObjectSchema),
    Tuple(TupleSchema<'a>),
    Newtype(NewtypeSchema<'a>),
}

impl<'a> ToTokens for Schema<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();

        let res = match self {
            Schema::Object(schema) => quote!(#zod::core::ast::ExportSchema::Object(#schema)),
            Schema::Tuple(schema) => quote!( #zod::core::ast::ExportSchema::Tuple(#schema)),
            Schema::Newtype(schema) => quote!(#zod::core::ast::ExportSchema::Newtype(#schema)),
        };

        tokens.extend(res)
    }
}

pub(super) struct ObjectSchema {
    pub fields: FilteredFields,
}

impl ToTokens for ObjectSchema {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let (ext, fields): (Vec<_>, Vec<_>) =
            self.fields.iter().cloned().partition(|f| f.config.flatten);

        let ext = ext.into_iter().map(|f| {
            let ty = f.ty;

            match f.config.derive {
                Derive::Request => quote!(#zod::core::ast::Ref::new_req::<#ty>()),
                Derive::Response => quote!(#zod::core::ast::Ref::new_res::<#ty>()),
            }
        });

        tokens.extend(
            quote!(#zod::core::ast::ObjectSchema::new(&[#(#fields),*]).with_extensions(&[#(#ext,)*])),
        );
    }
}

pub(super) struct NewtypeSchema<'a> {
    pub field: &'a Field,
}

impl<'a> ToTokens for NewtypeSchema<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let ty = &self.field.ty;

        let optional = if !self.field.config.required {
            quote!(.optional())
        } else {
            quote!()
        };

        let field = match self.field.generic {
            Some(ref ident) => {
                let name = ident.to_string();
                quote!(&#zod::core::ast::TupleField::generic(#name) #optional)
            }
            None => match self.field.config.derive {
                Derive::Request => {
                    quote!(&#zod::core::ast::TupleField::new_req::<#ty>() #optional)
                }
                Derive::Response => {
                    quote!(&#zod::core::ast::TupleField::new_res::<#ty>() #optional)
                }
            },
        };

        tokens.extend(quote! {
            #zod::core::ast::NewtypeSchema::new(#field)
        })
    }
}

pub(super) struct TupleSchema<'a> {
    fields: &'a FilteredFields,
}

impl<'a> TupleSchema<'a> {
    pub fn new(fields: &'a FilteredFields) -> Result<Self, Error> {
        if let Some(first_required) = fields.iter().position(|f| !f.config.required) {
            if let Some(err) = fields.iter().skip(first_required + 1).find_map(|f| {
                if f.config.required {
                    Some(Error::DefaultBeforeNonDefault(f.ty.span()))
                } else {
                    None
                }
            }) {
                return Err(err);
            }
        }
        Ok(Self { fields })
    }
}

impl<'a> ToTokens for TupleSchema<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let fields = self.fields;
        let fields = fields.iter();

        tokens.extend(quote! {
            #zod::core::ast::TupleSchema::new(&[#(#fields),*])
        })
    }
}

#[cfg(test)]
mod test {
    use syn::parse_quote;

    use crate::test_utils::compare;

    use super::*;

    #[test]
    fn tuple_schema_ok() {
        let input = TupleSchema {
            fields: &FilteredFields::new(
                vec![(&parse_quote!(T1), Default::default())],
                &[&parse_quote!(T1)],
            )
            .unwrap(),
        };

        compare(
            quote!(#input),
            quote!(::zod::core::ast::TupleSchema::new(&[
                ::zod::core::ast::TupleField::generic("T1")
            ])),
        )
    }
}
