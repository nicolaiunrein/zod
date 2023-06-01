mod r#enum;
mod fields;
mod generics;
mod r#struct;

use crate::utils::zod_core;
use crate::Kind;
use darling::FromDeriveInput;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use r#enum::EnumImpl;
use r#struct::StructImpl;
use serde_derive_internals::attr::TagType as SerdeTagType;
use syn::{parse_quote, DeriveInput};

use self::r#enum::TagType;

struct CustomSuffix {
    inner: Option<String>,
}

impl ToTokens for CustomSuffix {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let option = quote!(::std::option::Option);
        let expanded = match self.inner {
            Some(ref suffix) => quote!(#option::Some(::std::string::String::from(#suffix))),
            None => quote!(#option::None),
        };

        tokens.extend(expanded)
    }
}

enum Data {
    Struct(StructImpl),
    Enum(EnumImpl),
}

impl ToTokens for Data {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Data::Struct(inner) => inner.to_tokens(tokens),
            Data::Enum(inner) => inner.to_tokens(tokens),
        }
    }
}

impl Data {
    fn new(derive: Derive, input: syn::DeriveInput, tag: TagType) -> Self {
        match input.data {
            syn::Data::Struct(data) => {
                Self::Struct(StructImpl::new(derive, data.fields, &input.generics))
            }

            syn::Data::Enum(data) => {
                Self::Enum(EnumImpl::new(derive, data.variants, &input.generics, tag))
            }

            syn::Data::Union(_) => todo!("todo... not supported"),
        }
    }

    fn inline(&self) -> bool {
        match self {
            Data::Struct(inner) => inner.inline,
            Data::Enum(inner) => inner.inline,
        }
    }
}

impl From<&SerdeTagType> for TagType {
    fn from(value: &SerdeTagType) -> Self {
        match value {
            SerdeTagType::External => TagType::Externally,
            SerdeTagType::Internal { tag } => TagType::Internally {
                tag: tag.to_owned(),
            },
            SerdeTagType::Adjacent { tag, content } => TagType::Adjacently {
                tag: tag.to_owned(),
                content: content.to_owned(),
            },
            SerdeTagType::None => TagType::Untagged,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Copy)]
pub enum Derive {
    Input,
    Output,
}

impl ToTokens for Derive {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            Self::Input => tokens.extend(quote!(#zod_core::Kind::Input)),
            Self::Output => tokens.extend(quote!(#zod_core::Kind::Output)),
        }
    }
}

#[derive(FromDeriveInput)]
#[darling(
    attributes(zod),
    forward_attrs(allow, doc, cfg),
    supports(struct_named, struct_newtype, struct_tuple, enum_any)
)]

struct ZodAttrs {
    pub namespace: syn::Path,
    pub custom_suffix: Option<String>,
}

struct Ast {
    derive: Derive,
    pub ident: syn::Ident,
    pub data: Data,
    pub generics: syn::Generics,
    pub namespace: syn::Path,
    pub custom_suffix: CustomSuffix,
    pub name: String,
    pub tag: TagType,
    // pub transparent: bool,
    // pub default
    // pub type_from: Option<syn::Type>,
    // pub type_try_from: Option<syn::Type>,
    // pub type_into: Option<syn::Type>,
}

impl Ast {
    fn arg_idents(&self) -> Vec<&syn::Ident> {
        self.generics
            .params
            .iter()
            .map(|p| match p {
                syn::GenericParam::Lifetime(_) => todo!(),
                syn::GenericParam::Type(param) => &param.ident,
                syn::GenericParam::Const(_) => todo!(),
            })
            .collect()
    }

    fn args(&self) -> Vec<TokenStream2> {
        self.arg_idents()
            .iter()
            .map(|ident| {
                let name = ident.to_string();

                quote_spanned! {
                    ident.span() =>
                    #zod_core::GenericArgument::new::<#ident>(#name)
                }
            })
            .collect()
    }

    fn unique_ident(&self) -> syn::Ident {
        let name = &self.name;
        match self.derive {
            Derive::Input => {
                crate::utils::make_unique_name::<Kind::Input>(&quote::format_ident!("{name}"))
            }
            Derive::Output => {
                crate::utils::make_unique_name::<Kind::Output>(&quote::format_ident!("{name}"))
            }
        }
    }

    fn fix_where_clause(&mut self) {
        let derive = self.derive;
        if let Some(ref mut clause) = self.generics.where_clause {
            for p in clause.predicates.iter_mut() {
                match p {
                    syn::WherePredicate::Type(t) => {
                        t.bounds.push(syn::TypeParamBound::Trait(
                            parse_quote!(#zod_core::Type<#derive>),
                        ));
                    }
                    _ => {}
                }
            }
        } else {
            let predicates = self
                .arg_idents()
                .into_iter()
                .map(|ident| quote!(#ident: #zod_core::Type<#derive>));

            self.generics.where_clause = Some(parse_quote!(where #(#predicates),*))
        }
    }
}

impl Ast {
    pub fn new(derive: Derive, derive_input: DeriveInput) -> Result<Self, TokenStream2> {
        let cx = serde_derive_internals::Ctxt::new();
        let serde_attrs = serde_derive_internals::attr::Container::from_ast(&cx, &derive_input);
        cx.check().unwrap();

        let zod_attrs: ZodAttrs = match ZodAttrs::from_derive_input(&derive_input) {
            Ok(attrs) => attrs,
            Err(err) => return Err(err.write_errors()),
        };

        let mut this = Self {
            derive,
            ident: derive_input.ident.clone(),
            data: Data::new(derive, derive_input.clone(), serde_attrs.tag().into()),
            generics: derive_input.generics.clone(),
            namespace: zod_attrs.namespace,
            custom_suffix: CustomSuffix {
                inner: zod_attrs.custom_suffix,
            },
            name: match derive {
                Derive::Input => serde_attrs.name().deserialize_name(),
                Derive::Output => serde_attrs.name().serialize_name(),
            },
            tag: serde_attrs.tag().into(),
        };

        this.fix_where_clause();
        Ok(this)
    }
}

/// convert input into the generated code providing a `Derive`.
pub fn expand(derive: Derive, input: TokenStream2) -> TokenStream2 {
    let derive_input: DeriveInput = match syn::parse2(input) {
        Ok(parsed) => parsed,
        Err(err) => {
            return err.into_compile_error();
        }
    };

    let ast = match Ast::new(derive, derive_input) {
        Ok(attrs) => attrs,
        Err(err) => return err,
    };

    let ident = &ast.ident;
    let ns = &ast.namespace;
    let name = &ast.name;
    let custom_suffix = &ast.custom_suffix;
    let inline = ast.data.inline();
    let inner = &ast.data;
    let args = ast.args();
    let unique_ident = ast.unique_ident();

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    quote! {
        impl #impl_generics #zod_core::Type<#derive> for #ident #ty_generics #where_clause {
            type Ns = #ns;
            const NAME: &'static str = #name;
            const INLINE: bool = #inline;

            fn value() -> #zod_core::z::ZodType<#derive> {
                #zod_core::z::ZodType {
                    optional: false,
                    custom_suffix: #custom_suffix,
                    inner: #inner.into()
                }
            }

            fn args() -> ::std::vec::Vec<#zod_core::GenericArgument<#derive>> {
                let mut v = ::std::vec::Vec::new();
                #(v.push(#args);)*
                v
            }

            fn visit_dependencies(visitor: &mut #zod_core::DependencyVisitor<#derive>) {
                // TODO
            }
        }

        impl #ns {
            #[allow(dead_code)]
            #[allow(non_upper_case_globals)]
            const #unique_ident: () = {};
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::TokenStreamExt;
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::parse_quote;

    #[test]
    fn expand_zod_for_struct_with_named_fields_ok() {
        let derive = Derive::Input;
        let input = quote! {
            #[zod(namespace = "Ns")]
            struct Test {
                inner_string: String,
                inner_u8: u8
            }
        };

        let inner = StructImpl {
            fields: syn::Fields::Named(parse_quote!({ inner_string: String, inner_u8: u8 })),
            derive,
        };

        let custom_suffix = quote!(None);

        let expected = quote! {
            impl #zod_core::Type<#derive> for Test {
                type Ns = Ns;
                const NAME: &'static str = "Test";
                const INLINE: bool = false;

                fn value() -> #zod_core::z::ZodType<#derive> {
                    #zod_core::z::ZodType {
                        optional: false,
                        custom_suffix: #custom_suffix,
                        inner: #inner.into()
                    }
                }

                fn args() -> ::std::vec::Vec<#zod_core::GenericArgument<#derive>> {
                    let mut v = ::std::vec::Vec::new();
                    v
                }

                fn visit_dependencies(visitor: &mut #zod_core::DependencyVisitor<#zod_core::Kind::Input>) {}
            }

            impl Ns {
                const __ZOD_PRIVATE_INPUT___Test: () = {};
            }

        };

        assert_eq!(
            expand(Derive::Input, input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        )
    }

    #[test]
    fn expand_zod_for_struct_with_tuple_fields_ok() {
        let derive = Derive::Input;
        let input = quote! {
            #[zod(namespace = "Ns")]
            struct Test(String, u8);
        };

        let custom_suffix = quote!(None);

        let inner = StructImpl {
            fields: syn::Fields::Unnamed(parse_quote!((String, u8))),
            derive,
        };

        let expected = quote! {
            impl #zod_core::Type<#derive> for Test {
                type Ns = Ns;
                const NAME: &'static str = "Test";
                const INLINE: bool = false;

                fn value() -> #zod_core::z::ZodType<#derive> {
                    #zod_core::z::ZodType {
                        optional: false,
                        custom_suffix: #custom_suffix,
                        inner: #inner.into()
                    }
                }

                fn args() -> ::std::vec::Vec<#zod_core::GenericArgument<#derive>> {
                    let mut v = ::std::vec::Vec::new();
                    v
                }

                fn visit_dependencies(visitor: &mut #zod_core::DependencyVisitor<#zod_core::Kind::Input>) {}
            }

            impl Ns {
                const __ZOD_PRIVATE_INPUT___Test: () = {};
            }

        };

        assert_eq!(
            expand(Derive::Input, input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        )
    }

    #[test]
    fn impl_zod_for_enum() {
        let derive = Derive::Input;
        let input = quote! {
            #[zod(namespace = "Ns")]
            enum Test {
                Unit,
                Tuple1(String),
                Tuple2(String, u8),
                Struct0 {},
                Struct1 {
                    inner: String,
                },
                Struct2 {
                    inner_string: String,
                    inner_u8: u8,
                }
            }
        };

        let inner = EnumImpl {
            tag: Default::default(),
            derive,
            variants: vec![
                parse_quote!(Unit),
                parse_quote!(Tuple1(String)),
                parse_quote!(Tuple2(String, u8)),
                parse_quote!(Struct0 {}),
                parse_quote!(Struct1 { inner: String }),
                parse_quote!(Struct2 {
                    inner_string: String,
                    inner_u8: u8
                }),
            ],
        };

        let custom_suffix = quote!(None);

        let expected = quote! {
            impl #zod_core::Type<#derive> for Test {
                type Ns = Ns;
                const NAME: &'static str = "Test";
                const INLINE: bool = false;

                fn value() -> #zod_core::z::ZodType<#derive> {
                    #zod_core::z::ZodType {
                        optional: false,
                        custom_suffix: #custom_suffix,
                        inner: #inner.into()
                    }
                }

                fn args() -> ::std::vec::Vec<#zod_core::GenericArgument<#derive>> {
                    let mut v = ::std::vec::Vec::new();
                    v
                }

                fn visit_dependencies(visitor: &mut #zod_core::DependencyVisitor<#zod_core::Kind::Input>) {}
            }

            impl Ns {
                const __ZOD_PRIVATE_INPUT___Test: () = {};
            }

        };

        assert_eq!(
            expand(Derive::Input, input).to_formatted_string().unwrap(),
            expected.to_formatted_string().unwrap()
        );
    }
}
