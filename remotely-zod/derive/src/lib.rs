use darling::ast::Data;
use darling::ast::Fields;
use darling::FromDeriveInput;
use darling::FromField;
use darling::FromVariant;
use proc_macro::TokenStream;
use proc_macro_error::abort_call_site;
use proc_macro_error::proc_macro_error;
use quote::quote;
use quote::quote_spanned;
use syn::parse_macro_input;
use syn::spanned::Spanned;
use syn::Field;
use syn::Item;
use syn::ItemEnum;
use syn::ItemStruct;
use syn::Type;

#[derive(FromDeriveInput)]
#[darling(
    attributes(zod),
    forward_attrs(allow, doc, cfg),
    supports(struct_named, enum_named)
)]
struct Input {
    ident: syn::Ident,
    data: Data<EnumVariant, StructField>,
    attrs: Vec<syn::Attribute>,
    ns: String,
}

#[derive(FromVariant, Clone)]
struct EnumVariant {}

#[derive(FromField, Clone)]
struct StructField {
    ident: Option<syn::Ident>,
    ty: Type,
}

#[proc_macro_error]
#[proc_macro_derive(zod, attributes(zod))]
pub fn zod(input: TokenStream) -> TokenStream {
    let parsed = syn::parse(input).unwrap();
    let input = Input::from_derive_input(&parsed).unwrap();
    let expanded = match input.data.clone() {
        Data::Enum(e) => todo!(),
        Data::Struct(e) => expand_struct(input, e),
    };
    expanded.into()
}

fn expand_struct(input: Input, fields: Fields<StructField>) -> proc_macro2::TokenStream {
    let ident = input.ident;
    let ns = input.ns;
    let name = format!("{}.{}", ns, ident);
    let expanded_fields = fields
        .into_iter()
        .map(|StructField { ident, ty, .. }| match ident {
            Some(ident) => {
                let field_name = ident.to_string();
                quote_spanned! {ty.span() =>  format!("{}: {},", #field_name, #ty::compose()) }
            }
            None => todo!(),
        });

    quote! {
        impl remotely_zod::Codegen for #ident {
            fn code() -> String {
                let fields: Vec<String> = vec![#(#expanded_fields),*];

                format!("z.object({{{}}})", fields.join("\n"))
            }

            fn name() -> Option<&'static str> {
                Some(#name)
            }
        }
    }
}

fn expand_enum(e: ItemEnum) -> proc_macro2::TokenStream {
    quote!()
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn empty_struct() {
        let tt = syn::parse_str::<ItemStruct>("struct X {}").unwrap();
        let out = expand_struct(tt);

        let expected = quote! {
            impl remotely_zod::Codegen for X {
                fn code() -> String {
                    let fields: Vec<String> = vec![];
                    format!("z.object({{{}}})", fields.join("\n"))
                }

                fn name() -> Option<&'static str> {
                    Some("X")
                }
            }
        };

        assert_eq!(out.to_string(), expected.to_string());
    }

    #[test]
    fn struct_fields() {
        let s = quote! {
            struct X {
                a: usize,
                b: String
            }
        };

        let tt = syn::parse_str::<ItemStruct>(&s.to_string()).unwrap();
        let out = expand_struct(tt);

        let expected = quote! {
            impl remotely_zod::Codegen for X {
                fn code() -> String {
                    let fields: Vec<String> = vec![format!("{}: {},", "a", usize::compose()), format!("{}: {},", "b", String::compose())];
                    format!("z.object({{{}}})", fields.join("\n"))
                }

                fn name() -> Option<&'static str> {
                    Some("X")
                }
            }
        };

        assert_eq!(out.to_string(), expected.to_string());
    }
}
