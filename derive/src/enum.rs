use crate::config::ContainerConfig;
use crate::config::TagType;
use crate::utils::get_zod;
use proc_macro2::TokenStream;
use quote::quote;
use serde_derive_internals::ast::Style;
use serde_derive_internals::ast::Variant;

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

        let variants = self.variants.iter().map(|v| match v.style {
            Style::Struct => {
                let fields = v.fields.iter().map(|f| {
                    let ty = f.ty;
                    let name = match self.config.derive {
                        crate::config::Derive::Request => f.attrs.name().deserialize_name(),
                        crate::config::Derive::Response => f.attrs.name().serialize_name(),
                    };
                    quote!(#zod::core::ast::NamedField::#req_res::<#ty>(#name))
                });

                let variant_name = match self.config.derive {
                    crate::config::Derive::Request => v.attrs.name().deserialize_name(),
                    crate::config::Derive::Response => v.attrs.name().serialize_name(),
                };


                quote!(#zod::core::ast::Variant::ExternallyTagged(#variant_name, #option::Some(#zod::core::ast::VariantValue::Object(#zod::core::ast::ObjectSchema::new(&[
                    #(#fields),*
                ])))))
            }
            Style::Tuple => {

                let fields = v.fields.iter().map(|f| {
                    let ty = f.ty;
                    quote!(#zod::core::ast::TupleField::#req_res::<#ty>())
                });

                let variant_name = match self.config.derive {
                    crate::config::Derive::Request => v.attrs.name().deserialize_name(),
                    crate::config::Derive::Response => v.attrs.name().serialize_name(),
                };

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

                let variant_name = match self.config.derive {
                    crate::config::Derive::Request => v.attrs.name().deserialize_name(),
                    crate::config::Derive::Response => v.attrs.name().serialize_name(),
                };


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

                let variant_name = match self.config.derive {
                    crate::config::Derive::Request => v.attrs.name().deserialize_name(),
                    crate::config::Derive::Response => v.attrs.name().serialize_name(),
                };

                quote!(#zod::core::ast::Variant::ExternallyTagged(#variant_name, #option::None))

            }
        });

        let schema = match &self.config.tag {
            TagType::External => {
                quote!(#zod::core::ast::ExportSchema::Union(#zod::core::ast::UnionSchema::new(&[
                    #(#variants),*
                ])))
            }
            TagType::None => {
                quote!(#zod::core::ast::ExportSchema::Union(#zod::core::ast::UnionSchema::new(&[])))
            }
            TagType::Internal { tag } => {
                quote!(#zod::core::ast::ExportSchema::DiscriminatedUnion(#zod::core::ast::DiscriminatedUnionSchema::new(#tag, &[])))
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
