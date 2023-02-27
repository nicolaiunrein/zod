use crate::args::{get_rustdoc, EnumField};

use super::args;
use darling::ast::Fields;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use serde_derive_internals::{
    ast::{self, Data},
    attr::TagType,
};
use syn::{spanned::Spanned, Ident};

pub fn expand(
    input: args::Input,
    variants: Vec<args::EnumVariant>,
    serde_ast: ast::Container,
) -> TokenStream {
    Enum {
        input,
        variants,
        serde_ast,
    }
    .expand()
}

struct Enum<'a> {
    input: args::Input,
    variants: Vec<args::EnumVariant>,
    serde_ast: ast::Container<'a>,
}

impl<'a> Enum<'a> {
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
            Err(err) => err.into_compile_error(),
        }
    }

    fn variant_attrs(&self) -> Vec<&serde_derive_internals::attr::Variant> {
        match &self.serde_ast.data {
            Data::Enum(variants) => variants.into_iter().map(|v| &v.attrs).collect::<Vec<_>>(),
            Data::Struct(_, _) => unreachable!(),
        }
    }

    fn expand_one_variant(&self) -> TokenStream {
        let ident = &self.input.ident;
        let ident_str = ident.to_string();
        let ns_path = &self.input.namespace;
        let mut serde_variants = self.variant_attrs();

        let variant = self
            .variants
            .first()
            .map(|v| Variant::new(v, &self.serde_ast, serde_variants.pop().unwrap()))
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

        let variants = self
            .variants
            .iter()
            .zip(self.variant_attrs().into_iter())
            .map(|(v, attrs)| Variant::new(v, &self.serde_ast, attrs))
            .collect::<Vec<_>>();
        let expanded_variant_schemas = variants.iter().map(|v| v.expand_schema());
        let expanded_variant_type_defs = variants.iter().map(|v| v.expand_type_def());

        let docs = self.docs();

        let schema_def = match self.serde_ast.attrs.tag() {
            TagType::External => {
                quote! {
                    let variants: std::vec::Vec<String> = vec![#(#expanded_variant_schemas),*];
                    format!("z.union([{}])", variants.join(", "))
                }
            }
            TagType::Internal { tag } => {
                quote! {
                    let variants: std::vec::Vec<String> = vec![#(#expanded_variant_schemas),*];
                    format!("z.discriminatedUnion(\"{}\", [{}])", #tag, variants.join(", "))
                }
            }
            TagType::Adjacent { tag, .. } => {
                quote! {
                    let variants: std::vec::Vec<String> = vec![#(#expanded_variant_schemas),*];
                    format!("z.discriminatedUnion(\"{}\", [{}])", #tag, variants.join(", "))
                }
            }
            TagType::None => {
                quote! {
                    let variants: std::vec::Vec<String> = vec![#(#expanded_variant_schemas),*];
                    format!("z.union([{}])", variants.join(", "))
                }
            }
        };

        quote! {
            impl remotely_zod::Codegen for #ident {
                fn schema() -> String {
                    #schema_def
                }

                fn type_def() -> String {
                    let type_defs: std::vec::Vec<String> = vec![#(#expanded_variant_type_defs),*];
                    type_defs.join(" | ")
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

impl<'a> Variant<'a> {
    fn new(
        variant: &'a args::EnumVariant,
        serde_ast: &'a ast::Container,
        attrs: &'a serde_derive_internals::attr::Variant,
    ) -> Self {
        let ident = &variant.ident;
        let fields = VariantFields {
            fields: &variant.fields,
        };

        match variant.fields.style {
            darling::ast::Style::Unit => Self::Unit(UnitVariant {
                ident,
                serde_ast,
                attrs,
            }),
            darling::ast::Style::Tuple => Self::Tuple(TupleVariant {
                ident,
                fields,
                serde_ast,
                attrs,
            }),
            darling::ast::Style::Struct => Self::Struct(StructVariant {
                ident,
                fields,
                serde_ast,
                attrs,
            }),
        }
    }

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
    serde_ast: &'a ast::Container<'a>,
    attrs: &'a serde_derive_internals::attr::Variant,
}

impl<'a> UnitVariant<'a> {
    fn expand_schema(&self) -> TokenStream {
        let name = self.attrs.name().deserialize_name();
        match self.serde_ast.attrs.tag() {
            TagType::External => {
                quote_spanned!(self.ident.span() => format!("z.literal(\"{}\")", #name))
            }
            TagType::Internal { tag } => {
                quote_spanned!(self.ident.span() => format!("z.object({{ {}: z.literal(\"{}\") }})", #tag, #name))
            }
            TagType::Adjacent { tag, .. } => {
                quote_spanned!(self.ident.span() => format!("z.object({{ {}: z.literal(\"{}\") }})", #tag, #name))
            }
            TagType::None => {
                quote_spanned!(self.ident.span() => String::from("z.null()"))
            }
        }
    }

    /// Example `A`  ->  `"A"`
    fn expand_type_defs(&self) -> TokenStream {
        let name = self.attrs.name().deserialize_name();
        let span = self.ident.span();
        match self.serde_ast.attrs.tag() {
            TagType::External => {
                quote_spanned!(span => format!("\"{}\"", #name))
            }
            TagType::Internal { tag } => {
                quote_spanned!(span => format!("{{ {}: \"{}\" }}", #tag, #name))
            }
            TagType::Adjacent { tag, .. } => {
                quote_spanned!(span => format!("{{ {}: \"{}\" }}", #tag, #name))
            }
            TagType::None => {
                quote_spanned!(span => String::from("null"))
            }
        }
    }
}

/// represents a tuple variant of an enum, it has one or more unnamed fields. It is represented as a tuple in
/// zod which is a const array in typescript
struct TupleVariant<'a> {
    ident: &'a Ident,
    fields: VariantFields<'a>,
    serde_ast: &'a ast::Container<'a>,
    attrs: &'a serde_derive_internals::attr::Variant,
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
    /// Extern: `A(usize)  =>  z.object({ A: z.number().int().nonnegative() })`
    /// Intern: Unsupported
    /// Adjacent: `A(usize)  =>  z.object({ type: "A", content: z.number().int().nonnegative() })`
    /// Untagged: `A(usize)  =>  z.number().int().nonnegative()`
    fn expand_one_schema(&self) -> TokenStream {
        let inner = self.fields.expand_schema();
        let first = inner.first().unwrap();
        let name = self.attrs.name().deserialize_name();
        let span = self.ident.span();
        match self.serde_ast.attrs.tag() {
            TagType::External | TagType::Internal { .. } => {
                quote_spanned! {span =>  format!("z.object({{ {}: {} }})", #name, #first) }
            }
            TagType::Adjacent { tag, content } => {
                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {}: {} }})", #tag, #name, #content, #first) }
            }
            TagType::None => {
                quote_spanned! {span =>  String::from(#first) }
            }
        }
    }

    /// expand an enum tuple variant with more than one field into a zod schema
    /// Example: `A(usize, String)`  ->
    /// `z.object({ A: z.tuple([z.number().int().nonnegative(),  z.string()]) })`
    fn expand_n_schemas(&self) -> TokenStream {
        let inner = self.fields.expand_schema();
        let span = self.ident.span();
        let name = self.attrs.name().deserialize_name();

        let expanded_inner = quote! {
            {
                let inner: std::vec::Vec<String> = vec![#(#inner),*];
                format!("z.tuple([{}])", inner.join(", "))
            }
        };
        match self.serde_ast.attrs.tag() {
            TagType::External | TagType::Internal { .. } => {
                quote_spanned! {span =>  format!("z.object({{ {}: {} }})", #name, #expanded_inner) }
            }
            TagType::Adjacent { tag, content } => {
                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {}: {} }})", #tag, #name, #content, #expanded_inner) }
            }
            TagType::None => {
                quote_spanned! {span =>  String::from(#expanded_inner) }
            }
        }
    }

    fn expand_type_defs(&self) -> TokenStream {
        let expanded_fields = self.fields.expand_type_defs();
        let span = self.ident.span();
        let tag_type = self.serde_ast.attrs.tag();
        let name = self.attrs.name().deserialize_name();

        match expanded_fields.len() {
            // this case is handles by darling
            0 => unreachable!(),
            1 => {
                let first = expanded_fields.first().expect("exactly one variant");

                // expand an enum variant with exatly one field to a TS definition
                // External: `A(usize)` ->  `{ A: number }`
                // Adjacent: `A(usize)` ->  `{ type: "A", content: number }`
                match tag_type {
                    TagType::External | TagType::Internal { .. } => {
                        quote_spanned! {span =>  format!("{{ {}: {} }}", #name, #first) }
                    }
                    TagType::Adjacent { tag, content } => {
                        quote_spanned! {span =>  format!("{{ {}: \"{}\", {}: {} }}", #tag, #name, #content, #first) }
                    }
                    TagType::None => {
                        quote_spanned! {span =>  String::from(#first) }
                    }
                }
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

                match tag_type {
                    TagType::External | TagType::Internal { .. } => {
                        quote_spanned! {span =>  format!("{{ {}: {} }}", #name, #expanded_inner) }
                    }
                    TagType::Adjacent { tag, content } => {
                        quote_spanned! {span =>  format!("{{ {}: \"{}\", {}: {} }}", #tag, #name, #content, #expanded_inner) }
                    }
                    TagType::None => {
                        quote_spanned! {span => #expanded_inner }
                    }
                }
            }
        }
    }
}

/// represents a struct variant of an enum, it has one or more named fields. It is represeneted as
/// an object in typescript.
struct StructVariant<'a> {
    ident: &'a Ident,
    fields: VariantFields<'a>,
    serde_ast: &'a ast::Container<'a>,
    attrs: &'a serde_derive_internals::attr::Variant,
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
    /// External: `A{ num: usize } =>  z.object({ A: z.object({ num: z.number().int().nonnegative() }) })`
    /// Internal: `A{ num: usize } =>  z.object({ type: z.literal("A"), num: z.number().int().nonnegative() })`
    /// Adjacent: `A{ num: usize } =>  z.object({ type: z.literal("A"), content: z.object({ num: z.number().int().nonnegative() }) })`
    fn expand_one_field(&self) -> TokenStream {
        let inner = self.fields.expand_schema();
        let name = self.attrs.name().deserialize_name();

        let span = self.ident.span();
        let first = inner.first().unwrap();
        // let name = variant_names.first().unwrap();

        match self.serde_ast.attrs.tag() {
            TagType::External => {
                quote_spanned! {span =>  format!("z.object({{ {}: z.object({{ {} }}) }})", #name, #first) }
            }
            TagType::Internal { tag } => {
                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {} }})", #tag, #name, #first) }
            }
            TagType::Adjacent { tag, content } => {
                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {}: z.object({{ {} }}) }})", #tag, #name, #content, #first) }
            }
            TagType::None => {
                quote_spanned! {span =>  format!("z.object({{ {} }})", #first) }
            }
        }
    }

    /// expand an enum struct variant with more than one field into a zod schema
    /// External: `A{ num: usize, s: String}` ->
    /// `z.object({ A: z.object({ num: z.number().int().nonnegative(),  s: z.string()}) })`
    ///
    /// Internal: `A{ num: usize, s: String}` ->
    /// `z.object({ type: z.literal("A"), num: z.number().int().nonnegative(), s: z.string()})`
    ///
    /// Adjacent: `A{ num: usize, s: String}` ->
    /// `z.object({ type: z.literal("A"): content: z.object({ num: z.number().int().nonnegative(),  s: z.string()}) })`
    fn expand_many_fields(&self) -> TokenStream {
        let inner = self.fields.expand_schema();
        let span = self.ident.span();
        let name = self.attrs.name().deserialize_name();

        match self.serde_ast.attrs.tag() {
            TagType::External => {
                let expanded_inner = quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#inner),*];
                        inner.join(", ")
                    }
                };
                quote_spanned! {span =>  format!("z.object({{ {}: z.object({{ {} }}) }})", #name, #expanded_inner) }
            }
            TagType::Internal { tag } => {
                let expanded_inner = quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#inner),*];
                        inner.join(", ")
                    }
                };

                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {} }})", #tag, #name, #expanded_inner) }
            }
            TagType::Adjacent { tag, content } => {
                let expanded_inner = quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#inner),*];
                        inner.join(", ")
                    }
                };

                quote_spanned! {span =>  format!("z.object({{ {}: z.literal(\"{}\"), {}: z.object({{ {} }}) }})", #tag, #name, #content, #expanded_inner) }
            }
            TagType::None => {
                let expanded_inner = quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#inner),*];
                        inner.join(", ")
                    }
                };
                quote_spanned! {span =>  format!("z.object({{ {} }})", #expanded_inner) }
            }
        }
    }

    fn expand_type_defs(&self) -> TokenStream {
        let expanded_fields = self.fields.expand_type_defs();
        let span = self.ident.span();
        let name = self.attrs.name().deserialize_name();

        match expanded_fields.len() {
            // this case is handles by darling
            0 => unreachable!(),
            1 => {
                let first = expanded_fields.first().expect("exactly one variant");

                // expand an enum variant with exatly one field to a TS definition
                // External: `A{ num: usize }` ->  `{ A: { num: number }}`
                // Internal: `A{ num: usize }` ->  `{ type: "A", num: number }`
                // Adjacent: `A{ num: usize }` ->  `{ type: "A", content: { num: number }}`
                match self.serde_ast.attrs.tag() {
                    TagType::External => {
                        quote_spanned! {span =>  format!("{{ {}: {{ {} }} }}", #name, #first) }
                    }
                    TagType::Internal { tag } => {
                        quote_spanned! {span =>  format!("{{ {}: \"{}\", {} }}", #tag, #name, #first) }
                    }
                    TagType::Adjacent { tag, content } => {
                        quote_spanned! {span =>  format!("{{ {}: \"{}\", {}: {{ {} }} }}", #tag, #name, #content, #first) }
                    }
                    TagType::None => {
                        quote_spanned! {span =>  format!("{{ {} }}", #first) }
                    }
                }
            }

            // expand an enum tuple variant with more than one field to a TS definition
            // External: `A{ num: usize, s: String }` -> `{ A: { num: number, s: string } }`
            // Internal: `A{ num: usize, s: String }` -> `{ type: "A", num: number, s: string }`
            // Adjacent: `A{ num: usize, s: String }` -> `{ type: "A", content: { num: number, s: string }}`
            _ => {
                let expanded_inner = quote! {
                    {
                        let inner: std::vec::Vec<String> = vec![#(#expanded_fields),*];
                        inner.join(", ")
                    }
                };
                match self.serde_ast.attrs.tag() {
                    TagType::External => {
                        quote_spanned! {span =>  format!("{{ {}: {{ {} }} }}", #name, #expanded_inner) }
                    }
                    TagType::Internal { tag } => {
                        quote_spanned! {span =>  format!("{{ {}: \"{}\", {} }}", #tag, #name, #expanded_inner) }
                    }
                    TagType::Adjacent { tag, content } => {
                        quote_spanned! {span =>  format!("{{ {}: \"{}\", {}: {{ {} }} }}", #tag, #name, #content, #expanded_inner) }
                    }
                    TagType::None => {
                        quote_spanned! {span =>  format!("{{ {} }}", #expanded_inner) }
                    }
                }
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
