use darling::{
    ast::{Data, Fields},
    FromDeriveInput, FromField, FromVariant,
};
use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Type};

#[derive(FromDeriveInput)]
#[darling(
    attributes(zod),
    forward_attrs(allow, doc, cfg),
    supports(struct_named, enum_named)
)]
struct Input {
    ident: syn::Ident,
    data: Data<EnumVariant, StructField>,
    namespace: syn::Path,
}

#[derive(FromVariant, Clone)]
struct EnumVariant {}

#[derive(FromField, Clone)]
struct StructField {
    ident: Option<syn::Ident>,
    ty: Type,
}

#[proc_macro_derive(zod, attributes(zod))]
pub fn zod(input: TokenStream) -> TokenStream {
    let parsed = match syn::parse(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error().into();
        }
    };
    let input = match Input::from_derive_input(&parsed) {
        Ok(input) => input,
        Err(err) => {
            return err.write_errors().into();
        }
    };
    let expanded = match input.data.clone() {
        Data::Enum(e) => expand_enum(input, e),
        Data::Struct(e) => expand_struct(input, e),
    };
    expanded.into()
}

fn expand_struct(input: Input, fields: Fields<StructField>) -> proc_macro2::TokenStream {
    let ident = input.ident;
    let ident_str = ident.to_string();
    let ns_path = input.namespace.clone();

    let field_schemas = fields
        .iter()
        .map(|StructField { ident, ty, .. }| match ident {
            Some(ident) => {
                let field_name = ident.to_string();
                quote_spanned! {ty.span() =>  format!("{}: {},", #field_name, #ty::schema()) }
            }
            None => todo!(),
        });

    let field_type_defs = fields
        .iter()
        .map(|StructField { ident, ty, .. }| match ident {
            Some(ident) => {
                let field_name = ident.to_string();
                quote_spanned! {ty.span() =>  format!("{}: {},", #field_name, #ty::type_name()) }
            }
            None => todo!(),
        });

    quote! {
        impl remotely_zod::Codegen for #ident {
            fn schema() -> String {
                let fields: Vec<String> = vec![#(#field_schemas),*];
                format!("z.object({{{}}})", fields.join("\n"))
            }

            fn type_def() -> String {
                let fields: Vec<String> = vec![#(#field_type_defs),*];
                format!("{{{}}}", fields.join("\n"))
            }

            fn type_name() -> String {
                format!("{}.{}", <#ns_path as remotely::__private::codegen::namespace::Namespace>::NAME, #ident_str)
            }
        }
    }
}

fn expand_enum(_: Input, _: Vec<EnumVariant>) -> proc_macro2::TokenStream {
    todo!()
}
