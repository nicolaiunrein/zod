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
                String::from(#name)
            }
        }
    }
}

fn expand_enum(e: ItemEnum) -> proc_macro2::TokenStream {
    quote!()
}
