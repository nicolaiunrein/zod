use super::{
    r#struct::{ZodObjectImpl, ZodTupleImpl},
    Derive,
};
use crate::utils::zod_core;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote_spanned, ToTokens};
use syn::spanned::Spanned;

#[derive(Clone, Debug)]
pub(crate) enum FieldValue {
    Literal(String, Span),
    Type(syn::Type),
    Object(ZodObjectImpl),
    Tuple(ZodTupleImpl),
}

impl PartialEq for FieldValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FieldValue::Literal(a_value, a_span), FieldValue::Literal(b_value, b_span)) => {
                a_value == b_value
                    && a_span.start() == b_span.start()
                    && a_span.end() == b_span.end()
            }
            (FieldValue::Type(a), FieldValue::Type(b)) => a == b,
            _ => false,
        }
    }
}

impl From<syn::Type> for FieldValue {
    fn from(value: syn::Type) -> Self {
        Self::Type(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ZodNamedFieldImpl {
    pub name: String,
    pub optional: bool,
    pub derive: Derive,
    pub value: FieldValue,
}

impl ToTokens for ZodNamedFieldImpl {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let optional = &self.optional;
        let derive = self.derive;
        let (qualified_value, span) = match self.value {
            FieldValue::Literal(ref value, span) => (
                quote_spanned!(span => #zod_core::z::ZodLiteral::String(#value).into()),
                span,
            ),
            FieldValue::Type(ref ty) => (
                quote_spanned!(ty.span() => <#ty as #zod_core::TypeExt::<#derive>>::inline().into()),
                ty.span(),
            ),
            FieldValue::Object(ref obj) => (quote_spanned!(obj.span() => #obj.into()), obj.span()),
            FieldValue::Tuple(ref tuple) => {
                (quote_spanned!(tuple.span() => #tuple.into()), tuple.span())
            }
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
pub(crate) struct ZodUnnamedFieldImpl {
    pub derive: Derive,
    pub optional: bool,
    pub ty: syn::Type,
}

impl ToTokens for ZodUnnamedFieldImpl {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let optional = &self.optional;
        let ty = &self.ty;
        let derive = self.derive;
        let qualified_ty = quote_spanned!(ty.span() => <#ty as #zod_core::TypeExt::<#derive>>);

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
    use crate::test_utils::TokenStreamExt;

    use super::*;
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn expand_named_field_ok() {
        let derive = Derive::Input;
        let input = ZodNamedFieldImpl {
            name: String::from("hello"),
            derive,
            optional: false,
            value: FieldValue::Type(parse_quote!(String)),
        }
        .into_token_stream();

        let expected = quote! {
            #zod_core::z::ZodNamedField {
                name: "hello",
                optional: false,
                value: <String as #zod_core::TypeExt<#derive>>::inline().into()
            }
        };

        assert_eq!(
            input.to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        );
    }
}
