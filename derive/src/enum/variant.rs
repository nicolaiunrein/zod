use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use serde_derive_internals::ast::{self, Style};
use syn::spanned::Spanned;

use crate::config::ContainerConfig;
use crate::utils::get_zod;

pub(crate) struct Variant<'a>(&'a ast::Variant<'a>, &'a ContainerConfig);

impl<'a> Variant<'a> {
    pub fn new(v: &'a ast::Variant<'a>, config: &'a ContainerConfig) -> Self {
        Self(v, config)
    }

    pub(crate) fn skipped(&self) -> bool {
        match self.1.derive {
            crate::config::Derive::Request => self.0.attrs.skip_deserializing(),
            crate::config::Derive::Response => self.0.attrs.skip_serializing(),
        }
    }

    pub(crate) fn untagged(&self) -> TokenStream {
        match self.style() {
            Style::Struct => self.untagged_struct(),
            Style::Tuple => self.untagged_tuple(),
            Style::Newtype => self.untagged_newtype(),
            Style::Unit => self.untagged_unit(),
        }
    }

    pub(crate) fn internal(&self) -> TokenStream {
        match self.style() {
            Style::Struct | Style::Unit => self.internal_struct_or_unit(),
            Style::Newtype => self.internal_newtype(),
            Style::Tuple => unreachable!("prevented by serde"),
        }
    }

    pub(crate) fn adjacent(&self, content: &str) -> TokenStream {
        match self.style() {
            Style::Struct => self.adj_struct(content),
            Style::Tuple => self.adj_tuple(content),
            Style::Newtype => self.adj_newtype(content),
            Style::Unit => self.internal_struct_or_unit(),
        }
    }

    pub(crate) fn external(&self) -> TokenStream {
        match self.style() {
            Style::Struct => self.external_struct(),
            Style::Tuple => self.external_tuple(),
            Style::Newtype => self.external_newtype(),
            Style::Unit => self.external_unit(),
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

    fn named_fields<'b>(&'b self) -> impl Iterator<Item = TokenStream> + 'b {
        self.fields().map(|f| {
            let zod = get_zod();
            let req_res = self.1.req_or_res();
            let name = self.1.resolve_name(f.attrs.name());
            let ty = f.ty;

            let optional = if f.attrs.default().is_none() {
                quote!()
            } else {
                quote!(.optional())
            };

            quote!(#zod::core::ast::NamedField::#req_res::<#ty>(#name) #optional)
        })
    }

    fn tuple_fields<'b>(&'b self) -> impl Iterator<Item = TokenStream> + 'b {
        self.fields().map(|f| {
            let zod = get_zod();
            let ty = f.ty;
            let req_res = self.1.req_or_res();

            let optional = if f.attrs.default().is_none() {
                quote!()
            } else {
                quote!(.optional())
            };

            quote!(#zod::core::ast::TupleField::#req_res::<#ty>() #optional)
        })
    }

    fn untagged_tuple(&self) -> TokenStream {
        let zod = get_zod();
        let fields = self.tuple_fields();

        quote! {
            #zod::core::ast::Variant::Untagged(
                #zod::core::ast::VariantValue::Tuple(
                    #zod::core::ast::TupleSchema::new(&[#(#fields),*])
                )
            )
        }
    }

    fn untagged_struct(&self) -> TokenStream {
        let zod = get_zod();
        let fields = self.named_fields();

        quote! {
            #zod::core::ast::Variant::Untagged(
                #zod::core::ast::VariantValue::Object(
                    #zod::core::ast::ObjectSchema::new(&[#(#fields),*])
                )
            )
        }
    }

    fn untagged_newtype(&self) -> TokenStream {
        let zod = get_zod();

        let field = self.fields().next().expect("one field");
        let req_res = self.1.req_or_res();
        let ty = field.ty;

        let optional = !field.attrs.default().is_none();

        quote! {
            #zod::core::ast::Variant::Untagged(
                #zod::core::ast::VariantValue::Newtype(
                    #zod::core::ast::NewtypeSchema::new(
                        &#zod::core::ast::Ref::#req_res::<#ty>(), #optional
                    )
                )
            )
        }
    }

    fn untagged_unit(&self) -> TokenStream {
        let zod = get_zod();
        let req_res = self.1.req_or_res();

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

    pub(crate) fn internal_struct_or_unit(&self) -> TokenStream {
        let zod = get_zod();
        let fields = self.named_fields();
        let variant_name = self.1.resolve_name(self.name());

        quote! {
            #zod::core::ast::DiscriminatedVariant{
                tag: #variant_name,
                content_tag: ::core::option::Option::None,
                fields: &[#(#fields),*]
            }
        }
    }

    pub(crate) fn internal_newtype(&self) -> TokenStream {
        let zod = get_zod();
        if let Some(field) = self.fields().next() {
            let variant_name = self.1.resolve_name(self.name());
            let trait_name = self.1.trait_name();
            let ty = field.ty;

            let error = "zod: `internally tagged newtype variants are only supported for types compiling to zod objects.";

            let fields = quote_spanned! { self.span() =>
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
            self.internal_newtype()
        }
    }

    fn adj_struct(&self, content_tag: &str) -> TokenStream {
        let zod = get_zod();
        let fields = self.named_fields();
        let variant_name = self.1.resolve_name(self.name());

        quote! {
            #zod::core::ast::DiscriminatedVariant{
                tag: #variant_name,
                content_tag: ::core::option::Option::Some(#content_tag),
                fields: &[#(#fields),*]
            }
        }
    }

    fn adj_tuple(&self, content_tag: &str) -> TokenStream {
        let zod = get_zod();
        let req_res = self.1.req_or_res();
        let field_tys = self.fields().map(|f| f.ty.clone()).collect::<Vec<_>>();
        let field =
            quote!(#zod::core::ast::NamedField::#req_res::<(#(#field_tys),*,)>(#content_tag));

        let variant_name = self.1.resolve_name(self.name());

        quote! {
            #zod::core::ast::DiscriminatedVariant{
                tag: #variant_name,
                content_tag: ::core::option::Option::None,
                fields: &[#field]
            }
        }
    }

    fn adj_newtype(&self, content_tag: &str) -> TokenStream {
        let zod = get_zod();
        let req_res = self.1.req_or_res();
        let ty = self.fields().next().unwrap().ty;
        let field = quote!(#zod::core::ast::NamedField::#req_res::<#ty>(#content_tag));
        let variant_name = self.1.resolve_name(self.name());

        quote! {
            #zod::core::ast::DiscriminatedVariant{
                tag: #variant_name,
                content_tag: ::core::option::Option::None,
                fields: &[#field]
            }
        }
    }

    fn external_struct(&self) -> TokenStream {
        let zod = get_zod();
        let fields = self.named_fields();
        let variant_name = self.1.resolve_name(self.name());

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

    fn external_tuple(&self) -> TokenStream {
        let zod = get_zod();
        let fields = self.tuple_fields();
        let variant_name = self.1.resolve_name(self.name());

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

    fn external_newtype(&self) -> TokenStream {
        let zod = get_zod();
        if let Some(field) = self.fields().next() {
            let variant_name = self.1.resolve_name(self.name());
            let req_res = self.1.req_or_res();
            let ty = field.ty;
            let optional = !field.attrs.default().is_none();

            quote!(#zod::core::ast::Variant::ExternallyTagged(
                #variant_name,
                ::core::option::Option::Some(
                    #zod::core::ast::VariantValue::Newtype(
                        #zod::core::ast::NewtypeSchema::new(
                            &#zod::core::ast::Ref::#req_res::<#ty>(), #optional
                            )
                        )
                    )
                )
            )
        } else {
            self.external_unit()
        }
    }

    fn external_unit(&self) -> TokenStream {
        let zod = get_zod();
        let variant_name = self.1.resolve_name(self.name());
        quote!(#zod::core::ast::Variant::ExternallyTagged(#variant_name, ::core::option::Option::None))
    }
}
