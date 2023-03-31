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
    fn resolve_name(&self, name: &serde_derive_internals::attr::Name) -> String {
        match self.config.derive {
            crate::config::Derive::Request => name.deserialize_name(),
            crate::config::Derive::Response => name.serialize_name(),
        }
    }

    fn req_or_res(&self) -> TokenStream {
        match self.config.derive {
            crate::config::Derive::Request => quote!(new_req),
            crate::config::Derive::Response => quote!(new_res),
        }
    }

    fn external_struct(&self, v: &Variant) -> TokenStream {
        let zod = get_zod();

        let fields = v.fields.iter().map(|f| {
            let ty = f.ty;
            let name = self.resolve_name(f.attrs.name());
            let req_res = self.req_or_res();

            quote!(#zod::core::ast::NamedField::#req_res::<#ty>(#name))
        });

        let variant_name = self.resolve_name(v.attrs.name());

        quote!(#zod::core::ast::Variant::ExternallyTagged(
            #variant_name,
            ::core::option::Option::Some(
                #zod::core::ast::VariantValue::Object(
                    #zod::core::ast::ObjectSchema::new(&[#(#fields),*])
                    )
                )
            )
        )
    }

    fn external_tuple(&self, v: &Variant) -> TokenStream {
        let zod = get_zod();

        let fields = v.fields.iter().map(|f| {
            let ty = f.ty;
            let req_res = self.req_or_res();
            quote!(#zod::core::ast::TupleField::#req_res::<#ty>())
        });

        let variant_name = self.resolve_name(v.attrs.name());

        quote!(#zod::core::ast::Variant::ExternallyTagged(
            #variant_name,
            ::core::option::Option::Some(
                #zod::core::ast::VariantValue::Tuple(
                    #zod::core::ast::TupleSchema::new(&[#(#fields),*])
                    )
                )
            )
        )
    }

    fn external_newtype(&self, v: &Variant) -> TokenStream {
        let zod = get_zod();
        let field = v.fields.first().expect("one field");
        let variant_name = self.resolve_name(v.attrs.name());
        let req_res = self.req_or_res();
        let ty = field.ty;

        quote!(#zod::core::ast::Variant::ExternallyTagged(
            #variant_name,
            ::core::option::Option::Some(
                #zod::core::ast::VariantValue::Newtype(
                    #zod::core::ast::NewtypeSchema::new(
                        &#zod::core::ast::Ref::#req_res::<#ty>(), false
                        )
                    )
                )
            )
        )
    }

    fn external_unit(&self, v: &Variant) -> TokenStream {
        let zod = get_zod();
        let variant_name = self.resolve_name(v.attrs.name());
        quote!(#zod::core::ast::Variant::ExternallyTagged(#variant_name, ::core::option::Option::None))
    }

    fn internal_struct_or_unit(&self, v: &Variant) -> TokenStream {
        let zod = get_zod();
        let fields = v.fields.iter().map(|f| {
            let req_res = self.req_or_res();
            let name = self.resolve_name(f.attrs.name());
            let ty = f.ty;
            quote!(#zod::core::ast::NamedField::#req_res::<#ty>(#name))
        });

        let variant_name = self.resolve_name(v.attrs.name());

        quote! {
            #zod::core::ast::DiscriminatedVariant::InternallyTagged(#variant_name, &[
                #(#fields),*
            ])
        }
    }

    fn internal_newtype(&self, v: &Variant) -> TokenStream {
        let zod = get_zod();
        let field = v.fields.first().expect("one field");
        let variant_name = self.resolve_name(v.attrs.name());

        let trait_name = match self.config.derive {
            crate::config::Derive::Request => quote!(#zod::core::RequestType),
            crate::config::Derive::Response => quote!(#zod::core::ResponseType),
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

    pub(crate) fn expand(self) -> TokenStream {
        let zod = get_zod();
        let docs = &self.config.docs;
        let name = &self.config.name;
        let ns = &self.config.namespace;

        let schema = match &self.config.tag {
            // The default
            TagType::External => {
                let external_variants = self.variants.iter().map(|v| match v.style {
                    Style::Struct => self.external_struct(v),
                    Style::Tuple => self.external_tuple(v),
                    Style::Newtype => self.external_newtype(v),
                    Style::Unit => self.external_unit(v),
                });

                quote!(#zod::core::ast::ExportSchema::Union(#zod::core::ast::UnionSchema::new(&[
                    #(#external_variants),*
                ])))
            }

            TagType::Internal { tag } => {
                let variants = self.variants.iter().map(|v| match v.style {
                    Style::Struct | Style::Unit => self.internal_struct_or_unit(v),
                    Style::Newtype => self.internal_newtype(v),
                    Style::Tuple => unreachable!("prevented by serde"),
                });

                quote! {
                    #zod::core::ast::ExportSchema::DiscriminatedUnion(
                        #zod::core::ast::DiscriminatedUnionSchema::new(#tag, &[#(#variants),*]))
                }
            }

            // TODO
            // Untagged
            TagType::None => {
                quote!(#zod::core::ast::ExportSchema::Union(#zod::core::ast::UnionSchema::new(&[])))
            }

            // TODO
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
