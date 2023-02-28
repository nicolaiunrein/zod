mod field;
mod variant;

use crate::args::get_rustdoc;
use variant::Variant;

use super::args;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use serde_derive_internals::{
    ast::{self, Data},
    attr::TagType,
};

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
        let ident = &self.input.ident;
        let name = self.serde_ast.attrs.name().deserialize_name();
        let ns_path = &self.input.namespace;
        let schema = self.expand_schema();
        let type_def = self.expand_typ_defs();
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

    fn abort_empty(&self) -> ! {
        abort!(
            self.input.ident.span(),
            "deriving zod on empty enums is not supported."
        )
    }

    fn expand_schema(&self) -> TokenStream {
        match self.variants.len() {
            0 => self.abort_empty(),
            1 => self.variants().next().expect("one variant").expand_schema(),
            _ => {
                let expanded_variant_schemas = self.variants().map(|v| v.expand_schema());
                match self.serde_ast.attrs.tag() {
                    TagType::External => {
                        quote! {
                            let variants: std::vec::Vec<String> = vec![#(#expanded_variant_schemas),*];
                            format!("z.union([{}])", variants.join(", "))
                        }
                    }
                    TagType::Internal { tag } | TagType::Adjacent { tag, .. } => {
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
                }
            }
        }
    }

    fn expand_typ_defs(&self) -> TokenStream {
        match self.variants.len() {
            0 => self.abort_empty(),
            1 => self
                .variants()
                .next()
                .expect("one variant")
                .expand_type_def(),
            _ => {
                let expanded_variant_type_defs = self.variants().map(|v| v.expand_type_def());

                quote! {
                    let type_defs: std::vec::Vec<String> = vec![#(#expanded_variant_type_defs),*];
                    type_defs.join(" | ")
                }
            }
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

    fn variants(&'a self) -> impl Iterator<Item = Variant<'a>> {
        self.variants
            .iter()
            .zip(self.serde_variants().into_iter())
            .map(|(v, vars)| Variant::new(v, &self.serde_ast, vars))
    }
}
