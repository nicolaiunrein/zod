use crate::utils::get_zod;
use crate::zod_type::config::{ContainerConfig, TagType};
use darling::ToTokens;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
mod variant;
pub(crate) use variant::Variant;

#[allow(dead_code)]
pub(crate) struct EnumExport<'a> {
    pub(crate) variants: Vec<Variant<'a>>,
    pub(crate) config: &'a ContainerConfig,
    pub(crate) generics: &'a [Ident],
}

impl<'a> EnumExport<'a> {
    fn variants(&self) -> impl Iterator<Item = &Variant> {
        self.variants.iter().filter(|v| !v.skipped())
    }
}

impl<'a> ToTokens for EnumExport<'a> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let zod = get_zod();
        let docs = &self.config.docs;
        let name = &self.config.name;
        let ns = &self.config.namespace;

        let generics = self.generics.iter().map(|ident| ident.to_string());

        let schema = match &self.config.tag {
            // The default
            TagType::External => {
                let variants = self.variants().map(|v| v.external());
                quote! {
                    #zod::core::ast::ExportSchema::Union(
                        #zod::core::ast::UnionSchema::new(&[#(#variants),*], &[#(#generics),*])
                    )
                }
            }

            TagType::Internal { tag } => {
                let variants = self.variants().map(|v| v.internal());
                quote! {
                    #zod::core::ast::ExportSchema::DiscriminatedUnion(
                        #zod::core::ast::DiscriminatedUnionSchema::new(#tag, &[#(#variants),*], &[#(#generics),*]
                       ))
                }
            }

            TagType::None => {
                let variants = self.variants().map(|v| v.untagged());
                quote!(#zod::core::ast::ExportSchema::Union(#zod::core::ast::UnionSchema::new(&[
                    #(#variants),*
                ], &[
                    #(#generics),*
                ])))
            }

            TagType::Adjacent { tag, content } => {
                let variants = self.variants().map(|v| v.adjacent(content));
                quote! {
                    #zod::core::ast::ExportSchema::DiscriminatedUnion(
                        #zod::core::ast::DiscriminatedUnionSchema::new(#tag, &[#(#variants),*], &[
                            #(#generics),*
                        ])
                    )
                }
            }
        };

        tokens.extend(quote! {
            #zod::core::ast::Export {
                docs: #docs,
                path: #zod::core::ast::Path::new::<#ns>(#name),
                schema: #schema,
            }
        })
    }
}
