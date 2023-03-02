mod field;
mod variant;

use crate::{docs::RustDocs, expand_type_registration, impl_inventory};
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
    variants: &[args::EnumVariant],
    serde_ast: ast::Container,
    docs: RustDocs,
) -> TokenStream {
    let variant_ast = match serde_ast.data {
        Data::Enum(ref variants) => variants,
        Data::Struct(_, _) => unreachable!(),
    };

    let name = serde_ast.attrs.name().deserialize_name();
    let tag = serde_ast.attrs.tag();

    let variants = variants
        .iter()
        .zip(variant_ast.iter())
        .filter(|(_, ast)| !ast.attrs.skip_deserializing())
        .map(|(v, ast)| Variant::new(v, &serde_ast, ast))
        .collect();

    Enum {
        input,
        variants,
        name,
        tag,
        docs,
    }
    .expand()
}

struct Enum<'a> {
    input: args::Input,
    variants: Vec<Variant<'a>>,
    name: String,
    tag: &'a TagType,
    docs: RustDocs,
}

impl<'a> Enum<'a> {
    pub fn expand(&self) -> TokenStream {
        let ident = &self.input.ident;
        let name = &self.name;
        let ns_path = &self.input.namespace;
        let docs = &self.docs;
        let schema = self.expand_schema();
        let type_def = self.expand_typ_defs();

        let type_register = expand_type_registration(ident, ns_path);
        let inventory = impl_inventory::expand(ident, ns_path, name);

        quote! {
            impl ::zod::ZodType for #ident {
                fn schema() -> String {
                    #schema
                }

                fn type_def() -> ::zod::TsTypeDef {
                    ::zod::TsTypeDef::Type({ #type_def })
                }

                fn inline() -> ::zod::InlinedType {
                    ::zod::InlinedType::Ref {
                        ns_name: <#ns_path as ::zod::Namespace>::NAME,
                        name: #name
                    }
                }

                fn docs() -> Option<&'static str> {
                    Some(#docs)
                }
            }

            #inventory

            #type_register

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
            1 => self.variants.first().expect("one variant").expand_schema(),
            _ => {
                let expanded_variant_schemas = self.variants.iter().map(|v| v.expand_schema());

                match self.tag {
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
                .variants
                .first()
                .expect("one variant")
                .expand_type_def(),
            _ => {
                let expanded_variant_type_defs = self.variants.iter().map(|v| v.expand_type_def());

                quote! {
                    let type_defs: std::vec::Vec<String> = vec![#(#expanded_variant_type_defs),*];
                    type_defs.join(" | ")
                }
            }
        }
    }
}
