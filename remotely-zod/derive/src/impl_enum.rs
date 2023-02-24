use crate::args::{get_rustdoc, EnumField};

use super::args;
use darling::ast::Fields;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Ident};

pub fn expand(input: args::Input, variants: Vec<args::EnumVariant>) -> TokenStream {
    Enum { input, variants }.expand()
}

struct Enum {
    input: args::Input,
    variants: Vec<args::EnumVariant>,
}

impl Enum {
    pub fn expand(&self) -> TokenStream {
        match self.variants.len() {
            0 => abort!(
                self.input.ident.span(),
                "deriving zod on empty enums is not supported."
            ),
            1 => self.expand_one_variant(),
            _ => self.expand_many_variants(),
        }
    }

    fn docs(&self) -> TokenStream {
        match get_rustdoc(&self.input.attrs) {
            Ok(Some(docs)) => {
                let docs = format!(
                    "/**\n{}*/\n",
                    docs.lines()
                        .map(|line| format!("* {}\n", line))
                        .collect::<String>()
                );
                quote!(#docs)
            }
            Ok(None) => quote!(""),
            Err(err) => err.into_compile_error().into(),
        }
    }

    fn expand_one_variant(&self) -> TokenStream {
        let ident = &self.input.ident;
        let ident_str = ident.to_string();
        let ns_path = &self.input.namespace;

        let variant = self
            .variants
            .first()
            .map(Variant::from)
            .expect("one variant");

        let schema = variant.expand_schema();
        let type_def = variant.expand_type_def();
        let docs = self.docs();

        quote! {
            impl remotely_zod::Codegen for #ident {
                fn schema() -> String {
                    #schema
                }

                fn type_def() -> String {
                    #type_def
                }

                fn type_name() -> String {
                    format!("{}.{}", <#ns_path as ::remotely::__private::codegen::namespace::Namespace>::NAME, #ident_str)
                }

                fn docs() -> Option<&'static str> {
                    Some(#docs)
                }
            }
        }
    }

    fn expand_many_variants(&self) -> TokenStream {
        let ident = &self.input.ident;
        let ident_str = ident.to_string();
        let ns_path = self.input.namespace.clone();

        let variants = self.variants.iter().map(Variant::from).collect::<Vec<_>>();
        let expanded_variant_schemas = variants.iter().map(|v| v.expand_schema());
        let expanded_variant_type_defs = variants.iter().map(|v| v.expand_type_def());

        let docs = self.docs();

        quote! {
            impl remotely_zod::Codegen for #ident {
                fn schema() -> String {
                    let variants: std::vec::Vec<String> = vec![#(#expanded_variant_schemas),*];
                    format!("{}z.union([{}])", #docs, variants.join(", "))
                }

                fn type_def() -> String {
                    let type_defs: std::vec::Vec<String> = vec![#(#expanded_variant_type_defs),*];
                    format!("{}{}", #docs, type_defs.join(" | "))
                }

                fn type_name() -> String {
                    format!("{}.{}", <#ns_path as ::remotely::__private::codegen::namespace::Namespace>::NAME, #ident_str)
                }

                fn docs() -> Option<&'static str> {
                    Some(#docs)
                }
            }
        }
    }
}

/// represents a single enum variant
enum Variant<'a> {
    Unit(UnitVariant<'a>),
    Struct(StructVariant<'a>),
    Tuple(TupleVariant<'a>),
}

impl<'a> From<&'a args::EnumVariant> for Variant<'a> {
    fn from(variant: &'a args::EnumVariant) -> Self {
        let ident = &variant.ident;
        let fields = VariantFields {
            fields: &variant.fields,
        };

        match variant.fields.style {
            darling::ast::Style::Unit => Self::Unit(UnitVariant { ident }),
            darling::ast::Style::Tuple => Self::Tuple(TupleVariant { ident, fields }),
            darling::ast::Style::Struct => Self::Struct(StructVariant { ident, fields }),
        }
    }
}

impl<'a> Variant<'a> {
    /// expand a single variant of an enum into a zod schema
    fn expand_schema(&self) -> TokenStream {
        match self {
            Variant::Unit(unit) => unit.expand_schema(),
            Variant::Struct(strukt) => strukt.expand_schema(),
            Variant::Tuple(tuple) => tuple.expand_schema(),
        }
    }

    /// expand a single enum variant to TS definition
    fn expand_type_def(&self) -> TokenStream {
        match self {
            Variant::Unit(unit) => unit.expand_type_defs(),
            Variant::Struct(strukt) => strukt.expand_type_defs(),
            Variant::Tuple(tuple) => tuple.expand_type_defs(),
        }
    }
}

/// represents a unit variant of an enum, it has no fields and it is represented in typescript as the
/// stringifyied name
struct UnitVariant<'a> {
    ident: &'a Ident,
}

impl<'a> UnitVariant<'a> {
    fn expand_schema(&self) -> TokenStream {
        let ident_str = self.ident.to_string();
        quote_spanned!(self.ident.span() => format!("z.literal(\"{}\")", #ident_str))
    }

    /// Example `A`  ->  `"A"`
    fn expand_type_defs(&self) -> TokenStream {
        let ident_str = self.ident.to_string();
        let span = self.ident.span();
        quote_spanned!(span => format!("\"{}\"", #ident_str))
    }
}

/// represents a tuple variant of an enum, it has one or more unnamed fields. It is represented as a tuple in
/// zod which is a const array in typescript
struct TupleVariant<'a> {
    ident: &'a Ident,
    fields: VariantFields<'a>,
}

impl<'a> TupleVariant<'a> {
    fn expand_schema(&self) -> TokenStream {
        match self.fields.len() {
            // this case is handled by darling
            0 => unreachable!(),
            1 => self.expand_one_schema(),
            _ => self.expand_n_schemas(),
        }
    }

    /// expand an enum variant with exatly one field into a zod schema
    /// Example: `A(usize)  =>  z.object({ A: z.number().int().nonnegative() })`
    fn expand_one_schema(&self) -> TokenStream {
        let inner = self.fields.expand_schema();
        let first = inner.first().unwrap();
        let ident_str = self.ident.to_string();
        let span = self.ident.span();
        quote_spanned! {span =>  format!("z.object({{{}: {}}})", #ident_str, #first) }
    }

    /// expand an enum tuple variant with more than one field into a zod schema
    /// Example: `A(usize, String)`  ->
    /// `z.object({ A: z.tuple([z.number().int().nonnegative(),  z.string()]) })`
    fn expand_n_schemas(&self) -> TokenStream {
        let inner = self.fields.expand_schema();
        let span = self.ident.span();
        let ident_str = self.ident.to_string();

        let expanded_inner = quote! {
            {
                let inner: std::vec::Vec<String> = vec![#(#inner),*];
                format!("z.tuple([{}])", inner.join(", "))
            }
        };

        quote_spanned! {span =>  format!("z.object({{{}: {}}})", #ident_str, #expanded_inner) }
    }

    fn expand_type_defs(&self) -> TokenStream {
        let expanded_fields = self.fields.expand_type_defs();
        let span = self.ident.span();
        let ident_str = self.ident.to_string();

        match expanded_fields.len() {
            // this case is handles by darling
            0 => unreachable!(),
            1 => {
                let first = expanded_fields.first().expect("exactly one variant");

                // expand an enum variant with exatly one field to a TS definition
                // Example `A(usize)` ->  `{ A: number }`
                quote_spanned! {span =>  format!("{{ {}: {} }}", #ident_str, #first) }
            }

            // expand an enum tuple variant with more than one field to a TS definition
            // Example
            // `A(usize, String)` -> `{ A: [number, string] }`
            _ => {
                let expanded_inner = quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#expanded_fields),*];
                        format!("[{}]", inner.join(", "))
                    }
                };
                quote_spanned! {span =>  format!("{{ {}: {} }}", #ident_str, #expanded_inner) }
            }
        }
    }
}

/// represents a struct variant of an enum, it has one or more named fields. It is represeneted as
/// an object in typescript.
struct StructVariant<'a> {
    ident: &'a Ident,
    fields: VariantFields<'a>,
}

impl<'a> StructVariant<'a> {
    fn expand_schema(&self) -> TokenStream {
        match self.fields.len() {
            // this case is handled by darling
            0 => unreachable!(),
            1 => self.expand_one_field(),
            _ => self.expand_many_fields(),
        }
    }

    /// expand an enum variant with exatly one field into a zod schema
    /// Example: `A{ num: usize } =>  z.object({ A: z.object({ num: z.number().int().nonnegative() }) })`
    fn expand_one_field(&self) -> TokenStream {
        let inner = self.fields.expand_schema();
        let span = self.ident.span();
        let ident_str = self.ident.to_string();
        let first = inner.first().unwrap();
        quote_spanned! {span =>  format!("z.object({{{}: z.object({{ {} }}) }})", #ident_str, #first) }
    }

    /// expand an enum struct variant with more than one field into a zod schema
    /// Example: `A{ num: usize, s: String}` ->
    /// `z.object({ A: z.object({ num: z.number().int().nonnegative(),  s: z.string()}) })`
    fn expand_many_fields(&self) -> TokenStream {
        let inner = self.fields.expand_schema();
        let span = self.ident.span();
        let ident_str = self.ident.to_string();

        let expanded_inner = quote! {
            {
                let inner: std::vec::Vec<String> = vec![#(#inner),*];
                format!("z.object({{ {} }})", inner.join(", "))
            }
        };

        quote_spanned! {span =>  format!("z.object({{{}: {}}})", #ident_str, #expanded_inner) }
    }

    fn expand_type_defs(&self) -> TokenStream {
        let expanded_fields = self.fields.expand_type_defs();
        let span = self.ident.span();
        let ident_str = self.ident.to_string();

        match expanded_fields.len() {
            // this case is handles by darling
            0 => unreachable!(),
            1 => {
                let first = expanded_fields.first().expect("exactly one variant");

                // expand an enum variant with exatly one field to a TS definition
                // Example `A(usize)` ->  `{ A: number }`
                quote_spanned! {span =>  format!("{{ {}: {{ {} }} }}", #ident_str, #first) }
            }

            // expand an enum tuple variant with more than one field to a TS definition
            // Example
            // `A{ num: usize, s: String }` -> `{ A: { num: number, s: string } }`
            _ => {
                let expanded_inner = quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#expanded_fields),*];
                        format!("{}", inner.join(", "))
                    }
                };
                quote_spanned! {span =>  format!("{{ {}: {{ {} }} }}", #ident_str, #expanded_inner) }
            }
        }
    }
}

/// represents the fields inside a variant
struct VariantFields<'a> {
    fields: &'a Fields<EnumField>,
}

impl<'a> VariantFields<'a> {
    fn len(&self) -> usize {
        self.fields.len()
    }
    fn expand_type_defs(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|field| {
                let ty = &field.ty;
                let span = ty.span();
                match self.fields.style {
                    darling::ast::Style::Unit => unreachable!(),
                    darling::ast::Style::Tuple => {
                        quote_spanned!(span => format!("{}", <#ty as remotely_zod::Codegen>::type_def()))
                    }
                    darling::ast::Style::Struct => {
                        let name = field.ident.as_ref().unwrap().to_string();
                        quote_spanned!(span => format!("{}: {}", #name, <#ty as remotely_zod::Codegen>::type_def()))
                    }
                }
            })
            .collect()
    }

    fn expand_schema(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|field| {

                let ty = &field.ty;
                match self.fields.style {
                    darling::ast::Style::Unit => unreachable!(),
                    darling::ast::Style::Tuple => {
                        quote_spanned!(ty.span() => format!("{}", <#ty as remotely_zod::Codegen>::schema()))
                    }
                    darling::ast::Style::Struct => {
                        let ident_str = field.ident.as_ref().expect("named field").to_string();
                        quote_spanned!(ty.span() => format!("{}: {}", #ident_str, <#ty as remotely_zod::Codegen>::schema()))
                    }
                }

            })
            .collect()
    }
}
