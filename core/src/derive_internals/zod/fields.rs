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
    OverrideGetter(syn::Path),
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

impl FieldValue {
    fn expand(&self, derive: Derive) -> TokenStream2 {
        match self {
            FieldValue::Literal(ref value, ref span) => {
                quote_spanned!(*span => #zod_core::z::ZodLiteral::String(#value).into())
            }
            FieldValue::Type(ref ty) => {
                quote_spanned!(ty.span() => <#ty as #zod_core::TypeExt::<#derive>>::inline().into())
            }
            FieldValue::Object(ref obj) => quote_spanned!(obj.span() => #obj.into()),
            FieldValue::Tuple(ref tuple) => quote_spanned!(tuple.span() => #tuple.into()),
            FieldValue::OverrideGetter(ref p) => {
                quote_spanned!(p.span() => #zod_core::z::ZodType::from(#p()))
            }
        }
    }

    pub(super) fn span(&self) -> Span {
        match self {
            FieldValue::Literal(_, span) => span.clone(),
            FieldValue::Type(ty) => ty.span(),
            FieldValue::Object(ob) => ob.span(),
            FieldValue::Tuple(tuple) => tuple.span(),
            FieldValue::OverrideGetter(p) => p.span(),
        }
    }
}

impl ToTokens for ZodNamedFieldImpl {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let name = &self.name;
        let optional = &self.optional;
        let derive = self.derive;
        let value = self.value.expand(derive);
        let span = self.value.span();

        tokens.extend(quote_spanned! {
            span =>
            #zod_core::z::ZodNamedField {
                name: #name,
                optional: #optional,
                value: #value,
            }
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ZodUnnamedFieldImpl {
    pub derive: Derive,
    pub optional: bool,
    pub ty: FieldValue,
}

impl ToTokens for ZodUnnamedFieldImpl {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let optional = &self.optional;
        let ty = &self.ty;
        let derive = self.derive;
        let value = self.ty.expand(derive);

        tokens.extend(quote_spanned! {
            ty.span() =>
            #zod_core::z::ZodType {
                optional: #optional,
                ..#value
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
