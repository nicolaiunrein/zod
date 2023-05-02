use crate::config::Derive;
use crate::field::{Field, FilteredFields};
use crate::utils::get_zod;
use darling::ToTokens;
use proc_macro2::TokenStream;
use quote::quote;

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
        let optional = !self.field.config.required;

        let reference = match self.field.config.derive {
            Derive::Request => quote!(&#zod::core::ast::Ref::new_req::<#ty>()),
            Derive::Response => quote!(&#zod::core::ast::Ref::new_res::<#ty>()),
        };

        tokens.extend(quote! {
            #zod::core::ast::NewtypeSchema::new(#reference, #optional)
        })
    }
}

pub(super) struct TupleSchema<'a> {
    pub fields: &'a FilteredFields,
}

impl<'a> ToTokens for TupleSchema<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let fields = self.fields.iter();

        tokens.extend(quote! {
            #zod::core::ast::TupleSchema::new(&[#(#fields),*])
        })
    }
}
