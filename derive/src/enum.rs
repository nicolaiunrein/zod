use crate::config::ContainerConfig;
use crate::config::TagType;
use crate::utils::get_zod;
use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;
use serde_derive_internals::ast::Style;
use serde_derive_internals::ast::Variant;
use syn::spanned::Spanned;

#[allow(dead_code)]
pub(crate) struct EnumExport<'a> {
    pub(crate) variants: &'a [Variant<'a>],
    pub(crate) config: &'a ContainerConfig,
}

impl<'a> EnumExport<'a> {
    pub(crate) fn expand(self) -> TokenStream {
        let zod = get_zod();
        let docs = &self.config.docs;
        let name = &self.config.name;
        let ns = &self.config.namespace;

        let option = quote!(::core::option::Option);

        let req_res = match self.config.derive {
            crate::config::Derive::Request => quote!(new_req),
            crate::config::Derive::Response => quote!(new_res),
        };

        let get_name = |name: &serde_derive_internals::attr::Name| match self.config.derive {
            crate::config::Derive::Request => name.deserialize_name(),
            crate::config::Derive::Response => name.serialize_name(),
        };

        let external_variants = self.variants.iter().map(|v| match v.style {
            Style::Struct => {
                let fields = v.fields.iter().map(|f| {
                    let ty = f.ty;
                    let name = get_name(f.attrs.name());
                    quote!(#zod::core::ast::NamedField::#req_res::<#ty>(#name))
                });

                let variant_name = get_name(v.attrs.name());

                quote!(#zod::core::ast::Variant::ExternallyTagged(
                    #variant_name,
                    #option::Some(
                        #zod::core::ast::VariantValue::Object(
                            #zod::core::ast::ObjectSchema::new(&[#(#fields),*])
                            )
                        )
                    )
                )
            }
            Style::Tuple => {
                let fields = v.fields.iter().map(|f| {
                    let ty = f.ty;
                    quote!(#zod::core::ast::TupleField::#req_res::<#ty>())
                });

                let variant_name = get_name(v.attrs.name());

                quote!(#zod::core::ast::Variant::ExternallyTagged(
                    #variant_name,
                    #option::Some(
                        #zod::core::ast::VariantValue::Tuple(
                            #zod::core::ast::TupleSchema::new(&[#(#fields),*])
                            )
                        )
                    )
                )
            }
            Style::Newtype => {
                let field = v.fields.first().expect("one field");
                let variant_name = get_name(v.attrs.name());

                let ty = field.ty;

                quote!(#zod::core::ast::Variant::ExternallyTagged(
                    #variant_name,
                    #option::Some(
                        #zod::core::ast::VariantValue::Newtype(
                            #zod::core::ast::NewtypeSchema::new(
                                &#zod::core::ast::Ref::#req_res::<#ty>(), false
                                )
                            )
                        )
                    )
                )
            }
            Style::Unit => {
                let variant_name = get_name(v.attrs.name());
                quote!(#zod::core::ast::Variant::ExternallyTagged(#variant_name, #option::None))
            }
        });

        let schema = match &self.config.tag {
            // The default
            TagType::External => {
                quote!(#zod::core::ast::ExportSchema::Union(#zod::core::ast::UnionSchema::new(&[
                    #(#external_variants),*
                ])))
            }

            // Untagged
            TagType::None => {
                quote!(#zod::core::ast::ExportSchema::Union(#zod::core::ast::UnionSchema::new(&[])))
            }

            TagType::Internal { tag } => {
                let variants = self.variants.iter().map(|v| {
                    match v.style {
                        Style::Struct |
                        Style::Unit => {
                            let fields = v.fields.iter().map(|f| {
                                let name = get_name(f.attrs.name());
                                let ty = f.ty;
                                quote!(#zod::core::ast::NamedField::#req_res::<#ty>(#name))
                            });

                            let variant_name = get_name(v.attrs.name());

                            quote! {
                                #zod::core::ast::DiscriminatedVariant::InternallyTagged(#variant_name, &[
                                    #(#fields),*
                                ])
                            }
                        }
                        Style::Newtype => {
                            let field = v.fields.first().expect("one field");
                            let variant_name = get_name(v.attrs.name());

                            let trait_name = match self.config.derive {
                                crate::config::Derive::Request => quote!(#zod::core::RequestType),
                                crate::config::Derive::Response => quote!(#zod::core::ResponseType)
                            };

                            let ty = field.ty;
                            let error = "zod: `internally tagged newtype variants are only supported for types compiling to zod objects.";

                            let fields = quote_spanned! { v.original.span() => 
                                {
                                    const FIELDS: &'static [#zod::core::ast::NamedField] = 
                                        match <#ty as #trait_name>::EXPORT.schema {
                                            #zod::core::ast::ExportSchema::Object(schema) => schema.fields(),
                                            _ => panic!(#error)
                                        };
                                    FIELDS 
                                }
                             };

                            quote! {
                                #zod::core::ast::DiscriminatedVariant::InternallyTagged(#variant_name, #fields)
                            }
                        }
                        Style::Tuple => unreachable!("prevented by serde")
                    }
                });

                quote! {
                    #zod::core::ast::ExportSchema::DiscriminatedUnion(
                        #zod::core::ast::DiscriminatedUnionSchema::new(#tag, &[#(#variants),*]))
                }
            }
            TagType::Adjacent { tag, content } => {
                quote!(#zod::core::ast::ExportSchema::DiscriminatedUnion(#zod::core::ast::DiscriminatedUnionSchema::new(#tag, &[])))
            }
        };

        quote! {
            #zod::core::ast::Export {
                docs: #docs,
                path: #zod::core::ast::Path::new::<#ns>(#name),
                schema: #schema,
            }
        }
    }
}
