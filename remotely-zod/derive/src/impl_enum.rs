use crate::args::{get_rustdoc, EnumField};

use super::args;
use darling::ast::{Fields, Style};
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
    let variant_ast = match serde_ast.data {
        Data::Enum(ref variants) => variants,
        Data::Struct(_, _) => unreachable!(),
    };

    let variants = variants
        .into_iter()
        .zip(variant_ast.iter())
        .filter(|(_, ast)| !ast.attrs.skip_deserializing())
        .map(|(var, _)| var)
        .collect();

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

    fn serde_variants(&'a self) -> impl Iterator<Item = &serde_derive_internals::ast::Variant<'a>> {
        match &self.serde_ast.data {
            Data::Enum(variants) => variants.iter().filter(|v| !v.attrs.skip_deserializing()),
            Data::Struct(_, _) => unreachable!(),
        }
    }

    fn expand_one_variant(&self) -> TokenStream {
        let ident = &self.input.ident;
        let name = self.serde_ast.attrs.name().deserialize_name();
        let ns_path = &self.input.namespace;
        let serde_variants = self.serde_variants().collect::<Vec<_>>();

        let variant = self
            .variants
            .first()
            .map(|v| Variant::new(v, &self.serde_ast, serde_variants.first().unwrap()))
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
                    format!("{}.{}", <#ns_path as ::remotely::__private::codegen::namespace::Namespace>::NAME, #name)
                }

                fn docs() -> Option<&'static str> {
                    Some(#docs)
                }
            }
        }
    }

    fn expand_many_variants(&self) -> TokenStream {
        let ident = &self.input.ident;
        let name = self.serde_ast.attrs.name().deserialize_name();
        let ns_path = self.input.namespace.clone();

        let variants = self
            .variants
            .iter()
            .zip(self.serde_variants().into_iter())
            .map(|(v, vars)| Variant::new(v, &self.serde_ast, vars))
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
                    format!("{}.{}", <#ns_path as ::remotely::__private::codegen::namespace::Namespace>::NAME, #name)
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
        serde_variant: &'a serde_derive_internals::ast::Variant,
    ) -> Self {
        let ident = &variant.ident;
        let fields = VariantFields {
            all_fields: &variant.fields,
            serde_variant,
        };

        match variant.fields.style {
            darling::ast::Style::Unit => Self::Unit(UnitVariant {
                ident,
                serde_ast,
                attrs: &serde_variant.attrs,
            }),
            darling::ast::Style::Tuple => Self::Tuple(TupleVariant {
                ident,
                fields,
                serde_ast,
                attrs: &serde_variant.attrs,
            }),
            darling::ast::Style::Struct => Self::Struct(StructVariant {
                ident,
                fields,
                serde_ast,
                attrs: &serde_variant.attrs,
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
            // may occur if fields are skipped. In this case we handle it like a unit variant
            0 => UnitVariant {
                ident: self.ident,
                serde_ast: self.serde_ast,
                attrs: self.attrs,
            }
            .expand_schema(),
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
            // may occur if fields are skipped. In this case we handle it like a unit variant
            0 => UnitVariant {
                ident: self.ident,
                serde_ast: self.serde_ast,
                attrs: self.attrs,
            }
            .expand_type_defs(),
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
            0 => unreachable!("5"),
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
            0 => unreachable!("7"),
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

struct VariantField<'a> {
    enum_field: &'a EnumField,
    name: String,
    optional: bool,
}

/// represents the fields inside a variant
struct VariantFields<'a> {
    all_fields: &'a Fields<EnumField>,
    serde_variant: &'a ast::Variant<'a>,
}

impl<'a> VariantFields<'a> {
    fn len(&self) -> usize {
        self.fields().count()
    }

    fn fields(&self) -> impl Iterator<Item = VariantField> {
        self.all_fields
            .iter()
            .zip(&self.serde_variant.fields)
            .filter_map(|(enum_field, f)| {
                if !f.attrs.skip_deserializing() {
                    Some(VariantField {
                        enum_field,
                        name: f.attrs.name().deserialize_name(),
                        optional: !f.attrs.default().is_none(),
                    })
                } else {
                    None
                }
            })
    }

    fn style(&self) -> Style {
        self.all_fields.style
    }

    fn expand_type_defs(&self) -> Vec<TokenStream> {
        self.fields()
            .map(|f| {
                let ty = &f.enum_field.ty;
                let span = f.enum_field.ty.span();
                let name = f.name;

                match (self.style(), f.optional) {
                    (darling::ast::Style::Unit, _) => unreachable!("1"),
                    (darling::ast::Style::Tuple, false) => {
                        quote_spanned!(span => format!("{}", <#ty as remotely_zod::Codegen>::type_def()))
                    }
                    (darling::ast::Style::Struct, false) => {
                        quote_spanned!(span => format!("{}: {}", #name, <#ty as remotely_zod::Codegen>::type_def()))
                    }
                    (darling::ast::Style::Tuple, true) => {
                        quote_spanned!(span => format!("{} | undefined", <#ty as remotely_zod::Codegen>::type_def()))
                    }
                    (darling::ast::Style::Struct, true) => {
                        quote_spanned!(span => format!("{}?: {} | undefined", #name, <#ty as remotely_zod::Codegen>::type_def()))
                    }
                }
            })
            .collect()
    }

    fn expand_schema(&self) -> Vec<TokenStream> {
        self.fields()
            .map(|f| {
                let name = f.name;
                let ty = &f.enum_field.ty;
                match (self.style(), f.optional) {
                    (darling::ast::Style::Unit, _) => unreachable!("2"),
                    (darling::ast::Style::Tuple, false) => {
                        quote_spanned!(ty.span() => format!("{}", <#ty as remotely_zod::Codegen>::schema()))
                    }
                    (darling::ast::Style::Struct, false) => {
                        quote_spanned!(ty.span() => format!("{}: {}", #name, <#ty as remotely_zod::Codegen>::schema()))
                    }
                    (darling::ast::Style::Tuple, true) => {
                        quote_spanned!(ty.span() => format!("{}.optional()", <#ty as remotely_zod::Codegen>::schema()))
                    }
                    (darling::ast::Style::Struct, true) => {
                        quote_spanned!(ty.span() => format!("{}: {}.optional()", #name, <#ty as remotely_zod::Codegen>::schema()))
                    }
                }

            })
            .collect()
    }
}
