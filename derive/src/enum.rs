use crate::config::ContainerConfig;
use crate::config::TagType;
use crate::utils::get_zod;
use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::quote;
use quote::quote_spanned;
use serde_derive_internals::ast::Style;
use serde_derive_internals::ast::Variant;
use syn::spanned::Spanned;

pub(crate) struct MyVariant<'a>(&'a Variant<'a>, &'a ContainerConfig);

impl<'a> MyVariant<'a> {
    pub fn new(v: &'a Variant<'a>, config: &'a ContainerConfig) -> Self {
        Self(v, config)
    }

    fn skipped(&self) -> bool {
        match self.1.derive {
            crate::config::Derive::Request => self.0.attrs.skip_deserializing(),
            crate::config::Derive::Response => self.0.attrs.skip_serializing(),
        }
    }

    fn fields(&self) -> impl Iterator<Item = &serde_derive_internals::ast::Field> {
        self.0.fields.iter().filter(|f| !match self.1.derive {
            crate::config::Derive::Request => f.attrs.skip_deserializing(),
            crate::config::Derive::Response => f.attrs.skip_serializing(),
        })
    }

    fn name(&self) -> &serde_derive_internals::attr::Name {
        &self.0.attrs.name()
    }

    fn span(&self) -> Span {
        self.0.original.span()
    }

    fn style(&self) -> Style {
        self.0.style
    }
}

#[allow(dead_code)]
pub(crate) struct EnumExport<'a> {
    pub(crate) variants: Vec<MyVariant<'a>>,
    pub(crate) config: &'a ContainerConfig,
}

impl<'a> EnumExport<'a> {
    fn variants(&self) -> impl Iterator<Item = &MyVariant> {
        self.variants.iter().filter(|v| !v.skipped())
    }

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

    fn external_struct(&self, v: &MyVariant) -> TokenStream {
        let zod = get_zod();

        let fields = v.fields().map(|f| {
            let ty = f.ty;
            let name = self.resolve_name(f.attrs.name());
            let req_res = self.req_or_res();

            quote!(#zod::core::ast::NamedField::#req_res::<#ty>(#name))
        });

        let variant_name = self.resolve_name(v.0.attrs.name());

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

    fn external_tuple(&self, v: &MyVariant) -> TokenStream {
        let zod = get_zod();

        let fields = v.fields().map(|f| {
            let ty = f.ty;
            let req_res = self.req_or_res();
            quote!(#zod::core::ast::TupleField::#req_res::<#ty>())
        });

        let variant_name = self.resolve_name(v.0.attrs.name());

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

    fn external_newtype(&self, v: &MyVariant) -> TokenStream {
        let zod = get_zod();
        if let Some(field) = v.fields().next() {
            let variant_name = self.resolve_name(v.name());
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
        } else {
            self.external_unit(v)
        }
    }

    fn external_unit(&self, v: &MyVariant) -> TokenStream {
        let zod = get_zod();
        let variant_name = self.resolve_name(v.name());
        quote!(#zod::core::ast::Variant::ExternallyTagged(#variant_name, ::core::option::Option::None))
    }

    fn internal_struct_or_unit(&self, v: &MyVariant) -> TokenStream {
        let zod = get_zod();
        let fields = v.fields().map(|f| {
            let req_res = self.req_or_res();
            let name = self.resolve_name(f.attrs.name());
            let ty = f.ty;
            quote!(#zod::core::ast::NamedField::#req_res::<#ty>(#name))
        });

        let variant_name = self.resolve_name(v.name());

        quote! {
            #zod::core::ast::DiscriminatedVariant{
                tag: #variant_name,
                content_tag: ::core::option::Option::None,
                fields: &[#(#fields),*]
            }
        }
    }

    fn internal_newtype(&self, v: &MyVariant) -> TokenStream {
        let zod = get_zod();
        if let Some(field) = v.fields().next() {
            let variant_name = self.resolve_name(v.name());

            let trait_name = match self.config.derive {
                crate::config::Derive::Request => quote!(#zod::core::RequestType),
                crate::config::Derive::Response => quote!(#zod::core::ResponseType),
            };

            let ty = field.ty;
            let error = "zod: `internally tagged newtype variants are only supported for types compiling to zod objects.";

            let fields = quote_spanned! { v.span() =>
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
                #zod::core::ast::DiscriminatedVariant{
                    tag: #variant_name,
                    content_tag: ::core::option::Option::None,
                    fields: #fields
                }
            }
        } else {
            self.internal_newtype(v)
        }
    }

    fn adj_struct(&self, v: &MyVariant, content_tag: &str) -> TokenStream {
        let zod = get_zod();
        let fields = v.fields().map(|f| {
            let req_res = self.req_or_res();
            let name = self.resolve_name(f.attrs.name());
            let ty = f.ty;
            quote!(#zod::core::ast::NamedField::#req_res::<#ty>(#name))
        });

        let variant_name = self.resolve_name(v.name());

        quote! {
            #zod::core::ast::DiscriminatedVariant{
                tag: #variant_name,
                content_tag: ::core::option::Option::Some(#content_tag),
                fields: &[#(#fields),*]
            }
        }
    }

    fn adj_tuple(&self, v: &MyVariant, content_tag: &str) -> TokenStream {
        let zod = get_zod();

        let req_res = self.req_or_res();

        let fields = v.fields().map(|f| f.ty.clone()).collect::<Vec<_>>();

        let field = quote!(#zod::core::ast::NamedField::#req_res::<(#(#fields),*,)>(#content_tag));

        let variant_name = self.resolve_name(v.name());

        quote! {
            #zod::core::ast::DiscriminatedVariant{
                tag: #variant_name,
                content_tag: ::core::option::Option::None,
                fields: &[#field]
            }
        }
    }

    fn adj_newtype(&self, v: &MyVariant, content_tag: &str) -> TokenStream {
        let zod = get_zod();
        let req_res = self.req_or_res();
        let ty = v.fields().next().unwrap().ty;
        let field = quote!(#zod::core::ast::NamedField::#req_res::<#ty>(#content_tag));
        let variant_name = self.resolve_name(v.name());

        quote! {
            #zod::core::ast::DiscriminatedVariant{
                tag: #variant_name,
                content_tag: ::core::option::Option::None,
                fields: &[#field]
            }
        }
    }

    fn untagged_struct(&self, v: &MyVariant) -> TokenStream {
        let zod = get_zod();

        let fields = v.fields().map(|f| {
            let req_res = self.req_or_res();
            let name = self.resolve_name(f.attrs.name());
            let ty = f.ty;
            quote!(#zod::core::ast::NamedField::#req_res::<#ty>(#name))
        });

        quote! {
            #zod::core::ast::Variant::Untagged(
                #zod::core::ast::VariantValue::Object(
                    #zod::core::ast::ObjectSchema::new(&[#(#fields),*])
                )
            )
        }
    }

    fn untagged_tuple(&self, v: &MyVariant) -> TokenStream {
        let zod = get_zod();

        let fields = v.fields().map(|f| {
            let req_res = self.req_or_res();
            let ty = f.ty;
            quote!(#zod::core::ast::TupleField::#req_res::<#ty>())
        });

        quote! {
            #zod::core::ast::Variant::Untagged(
                #zod::core::ast::VariantValue::Tuple(
                    #zod::core::ast::TupleSchema::new(&[#(#fields),*])
                )
            )
        }
    }

    fn untagged_newtype(&self, v: &MyVariant) -> TokenStream {
        let zod = get_zod();

        let field = v.fields().next().expect("one field");
        let req_res = self.req_or_res();
        let ty = field.ty;

        quote! {
            #zod::core::ast::Variant::Untagged(
                #zod::core::ast::VariantValue::Newtype(
                    #zod::core::ast::NewtypeSchema::new(
                        &#zod::core::ast::Ref::#req_res::<#ty>(), false
                    )
                )
            )
        }
    }

    fn untagged_unit(&self) -> TokenStream {
        let zod = get_zod();

        let req_res = self.req_or_res();

        quote! {
            #zod::core::ast::Variant::Untagged(
                #zod::core::ast::VariantValue::Newtype(
                    #zod::core::ast::NewtypeSchema::new(
                        &#zod::core::ast::Ref::#req_res::<()>(), false
                    )
                )
            )
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
                let variants = self.variants().map(|v| match v.style() {
                    Style::Struct => self.external_struct(v),
                    Style::Tuple => self.external_tuple(v),
                    Style::Newtype => self.external_newtype(v),
                    Style::Unit => self.external_unit(v),
                });

                quote! {
                    #zod::core::ast::ExportSchema::Union(
                        #zod::core::ast::UnionSchema::new(&[#(#variants),*])
                    )
                }
            }

            TagType::Internal { tag } => {
                let variants = self.variants().map(|v| match v.style() {
                    Style::Struct | Style::Unit => self.internal_struct_or_unit(v),
                    Style::Newtype => self.internal_newtype(v),
                    Style::Tuple => unreachable!("prevented by serde"),
                });

                quote! {
                    #zod::core::ast::ExportSchema::DiscriminatedUnion(
                        #zod::core::ast::DiscriminatedUnionSchema::new(#tag, &[#(#variants),*]))
                }
            }

            // Untagged
            TagType::None => {
                let variants = self.variants().map(|v| match v.style() {
                    Style::Struct => self.untagged_struct(v),
                    Style::Tuple => self.untagged_tuple(v),
                    Style::Newtype => self.untagged_newtype(v),
                    Style::Unit => self.untagged_unit(),
                });

                quote!(#zod::core::ast::ExportSchema::Union(#zod::core::ast::UnionSchema::new(&[
                    #(#variants),*
                ])))
            }

            // TODO
            TagType::Adjacent { tag, content } => {
                let variants = self.variants().map(|v| match v.style() {
                    Style::Struct => self.adj_struct(v, content),
                    Style::Tuple => self.adj_tuple(v, content),
                    Style::Newtype => self.adj_newtype(v, content),
                    Style::Unit => self.internal_struct_or_unit(v),
                });

                quote! {
                    #zod::core::ast::ExportSchema::DiscriminatedUnion(
                        #zod::core::ast::DiscriminatedUnionSchema::new(#tag, &[#(#variants),*])
                    )
                }
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
