use crate::utils::zod_core;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote_spanned, ToTokens};
use syn::spanned::Spanned;

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ZodNamedFieldImpl<Io> {
    pub name: String,
    pub optional: bool,
    pub kind: Io,
    pub ty: syn::Type,
}

impl<Io> ToTokens for ZodNamedFieldImpl<Io>
where
    Io: ToTokens + Copy,
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let optional = &self.optional;
        let kind = self.kind;
        let ty = &self.ty;
        let qualified_ty = quote_spanned!(ty.span() => <#ty as #zod_core::Type::<#kind>>);

        tokens.extend(quote_spanned! {
            ty.span() =>
            #zod_core::types::ZodNamedField {
                name: #name,
                optional: #optional,
                value: #qualified_ty::get_ref().into(),
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct ZodUnnamedFieldImpl<Io> {
    pub kind: Io,
    pub optional: bool,
    pub ty: syn::Type,
}

impl<Io> ToTokens for ZodUnnamedFieldImpl<Io>
where
    Io: ToTokens + Copy,
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let optional = &self.optional;
        let ty = &self.ty;
        let kind = self.kind;
        let qualified_ty = quote_spanned!(ty.span() => <#ty as #zod_core::Type::<#kind>>);

        tokens.extend(quote_spanned! {
            ty.span() =>
            #zod_core::types::ZodType {
                optional: #optional,
                ..#qualified_ty::get_ref().into()
            }
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{test_utils::TokenStreamExt, Kind};

    use super::*;
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn expand_named_field_ok() {
        let kind = Kind::Input;
        let input = ZodNamedFieldImpl {
            name: String::from("hello"),
            kind,
            optional: false,
            ty: parse_quote!(String),
        }
        .into_token_stream();

        let expected = quote! {
            #zod_core::types::ZodNamedField {
                name: "hello",
                optional: false,
                value: <String as #zod_core::Type<#kind>>::get_ref().into()
            }
        };

        assert_eq!(
            input.to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        );
    }
}
