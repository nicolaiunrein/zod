use crate::utils::zod_core;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote_spanned, ToTokens};
use syn::spanned::Spanned;

#[derive(Clone, Debug)]
pub(crate) enum FieldValue {
    Literal(String, Span),
    Type(syn::Type),
}

impl From<syn::Type> for FieldValue {
    fn from(value: syn::Type) -> Self {
        Self::Type(value)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ZodNamedFieldImpl<Io> {
    pub name: String,
    pub optional: bool,
    pub kind: Io,
    pub value: FieldValue,
}

impl<Io> ToTokens for ZodNamedFieldImpl<Io>
where
    Io: ToTokens + Copy,
{
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let optional = &self.optional;
        let kind = self.kind;
        let (qualified_value, span) = match self.value {
            FieldValue::Literal(ref value, span) => (
                quote_spanned!(span => #zod_core::z::ZodLiteral::String(#value).into()),
                span,
            ),
            FieldValue::Type(ref ty) => (
                quote_spanned!(ty.span() => <#ty as #zod_core::TypeExt::<#kind>>::inline().into()),
                ty.span(),
            ),
        };

        tokens.extend(quote_spanned! {
            span =>
            #zod_core::z::ZodNamedField {
                name: #name,
                optional: #optional,
                value: #qualified_value,
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
        let qualified_ty = quote_spanned!(ty.span() => <#ty as #zod_core::TypeExt::<#kind>>);

        tokens.extend(quote_spanned! {
            ty.span() =>
            #zod_core::z::ZodType {
                optional: #optional,
                ..#qualified_ty::inline().into()
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
            value: FieldValue::Type(parse_quote!(String)),
        }
        .into_token_stream();

        let expected = quote! {
            #zod_core::z::ZodNamedField {
                name: "hello",
                optional: false,
                value: <String as #zod_core::TypeExt<#kind>>::inline().into()
            }
        };

        assert_eq!(
            input.to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        );
    }
}
