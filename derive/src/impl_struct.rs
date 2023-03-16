use crate::{docs::RustDocs, get_zod};

use super::args;
use darling::ast::{Fields, Style};
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{quote, quote_spanned};
use serde_derive_internals::ast;
use syn::{spanned::Spanned, Ident, Path};

fn qualified_ty(ty: &syn::Type) -> proc_macro2::TokenStream {
    let zod = get_zod();
    quote!(<#ty as #zod::ZodType>)
}

pub fn expand(
    input: args::Input,
    fields: Fields<args::StructField>,
    serde_ast: ast::Container,
    docs: RustDocs,
) -> proc_macro2::TokenStream {
    let transparent = serde_ast.attrs.transparent();

    let ident = input.ident;
    let name = serde_ast.attrs.name().deserialize_name();
    let ns_path = input.namespace;

    let fields_ast = match serde_ast.data {
        ast::Data::Enum(_) => unreachable!(),
        ast::Data::Struct(_, fields) => fields,
    };

    let style = fields.style;

    let generic_params = input
        .generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Type(t) => Some(t.ident.clone()),
            syn::GenericParam::Lifetime(_) => None,
            syn::GenericParam::Const(_) => None,
        })
        .collect::<Vec<_>>();

    let fields = fields
        .iter()
        .zip(fields_ast.iter().map(|f| &f.attrs))
        .filter(|(_, attrs)| !attrs.skip_deserializing())
        .map(|(args::StructField { ty, ident }, attrs)| StructField {
            ty,
            name: ident.as_ref().map(|_| attrs.name().deserialize_name()),
            optional: !attrs.default().is_none(),
            transparent,
            flatten: attrs.flatten(),
            generic_params: &generic_params,
        })
        .collect();

    let from_ty = serde_ast
        .attrs
        .type_from()
        .or_else(|| serde_ast.attrs.type_try_from())
        .cloned();

    let struct_def = Struct {
        transparent,
        ns_path,
        name,
        docs,
        ident,
        fields,
        style,
        from_ty,
        generics: input.generics.clone(),
        generic_params: generic_params.clone(),
    };

    struct_def.expand()
}

struct Struct<'a> {
    transparent: bool,
    ident: Ident,
    ns_path: Path,
    docs: RustDocs,
    name: String,
    fields: Vec<StructField<'a>>,
    style: Style,
    from_ty: Option<syn::Type>,
    generics: syn::Generics,
    generic_params: Vec<Ident>,
}

impl<'a> Struct<'a> {
    fn expand(&self) -> TokenStream {
        let schema = self.expand_schema();
        let type_def = self.expand_type_def();
        let ident = &self.ident;
        let ns_path = &self.ns_path;
        let name = &self.name;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let zod = get_zod();

        let docs = &self.docs;
        let concatcp = quote!(#zod::__private::const_format::concatcp);

        quote! {
            const _: () = {
                const AST: #zod::Code = #zod::Code {
                    ns_name: <#ns_path as #zod::Namespace>::NAME,
                    name: #name,
                    type_def: #concatcp!(#docs, #type_def),
                    schema: #concatcp!(#docs, #schema),
                };

                impl #impl_generics #zod::ZodType for #ident #ty_generics #where_clause {
                    const AST: #zod::Code = AST;
                }

                #zod::__private::inventory::submit!(AST);
            };
        }
    }

    fn expand_type_def(&self) -> TokenStream {
        let (flat_fields, fields) = self.fields.iter().partition::<Vec<_>, _>(|f| f.flatten);
        let zod = get_zod();
        let formatcp = quote!(#zod::__private::const_format::formatcp);
        let concatcp = quote!(#zod::__private::const_format::concatcp);
        let name = &self.name;

        let generic_params = if self.generic_params.is_empty() {
            quote!("")
        } else {
            let names = self
                .generic_params
                .iter()
                .map(|ident| format!("{},", ident.to_string()));
            quote!(#formatcp!("<{}>", #concatcp!(#(#names),*)))
        };

        match (self.transparent, self.style) {
            (true, _) => {
                let def = fields
                    .into_iter()
                    .next()
                    .expect("At least one field")
                    .expand_type_defs();

                quote!(#formatcp!("export type {} = {};", #name, #def))
            }

            (false, Style::Tuple) => match fields.len() {
                0 => unreachable!("handled by darling"),
                1 => {
                    let type_def = fields
                        .into_iter()
                        .next()
                        .or_else(|| flat_fields.into_iter().next())
                        .expect("Newtype")
                        .expand_type_defs();

                    quote!(
                        #formatcp!("export type {} = {};", #name, #type_def)
                    )
                }
                _ => {
                    let len = fields.len();
                    let fields = fields.into_iter().enumerate().map(|(i, f)| {
                        let def = f.expand_type_defs();
                        if i + 1 == len {
                            def
                        } else {
                            quote!(#concatcp!(#def, ", "))
                        }
                    });

                    quote! {
                        #formatcp!("export type {}{} = [{}];", #name, #generic_params, #concatcp!(#(#fields), *))
                    }
                }
            },

            (false, Style::Struct) => {
                let fields = fields.into_iter().map(|f| f.expand_type_defs());

                let maybe_extends = if flat_fields.is_empty() {
                    quote!("")
                } else {
                    let len = flat_fields.len();
                    let flat_fields = flat_fields.into_iter().enumerate().map(|(i, f)| {
                        let tt = f.expand_type_defs();
                        if i + 1 == len {
                            quote!(#tt)
                        } else {
                            quote!(#concatcp!(#tt, ", "))
                        }
                    });

                    quote!(#formatcp!("extends {}", #concatcp!(#(#flat_fields)*,)))
                };

                quote! {
                    #formatcp!("export interface {}{} {} {{ \n{}}}", #name, #generic_params, #maybe_extends, #concatcp!(#(#concatcp!("  ", #fields, ",\n")),*))
                }
            }

            (false, Style::Unit) => unreachable!(),
        }
    }

    fn expand_schema(&self) -> TokenStream {
        let (flat_fields, fields) = self.fields.iter().partition::<Vec<_>, _>(|f| f.flatten);
        let zod = get_zod();
        let formatcp = quote!(#zod::__private::const_format::formatcp);
        let concatcp = quote!(#zod::__private::const_format::concatcp);
        let has_generics = !self.generic_params.is_empty();

        let generic_params = if self.generic_params.is_empty() {
            quote!("")
        } else {
            let names = self
                .generic_params
                .iter()
                .map(|ident| format!("{}: z.ZodTypeAny,", ident.to_string()));

            quote!(#concatcp!(#(#names),*))
        };

        let name = &self.name;

        match (self.transparent, self.style) {
            (true, _) => {
                let def = fields
                    .into_iter()
                    .next()
                    .expect("At least one field")
                    .expand_schema();

                quote!(#formatcp!("export const {} = z.lazy(() => {});", #name, #def))
            }
            (false, Style::Tuple) => match fields.len() {
                0 => unreachable!("handled by darling"),
                1 => {
                    let schema = fields
                        .into_iter()
                        .next()
                        .or_else(|| flat_fields.into_iter().next())
                        .expect("Newtype")
                        .expand_schema();

                    if has_generics {
                        quote! {
                            #formatcp!("export const {} = ({}) => z.lazy(() => {});", #name, #generic_params, #schema)
                        }
                    } else {
                        quote! {
                            #formatcp!("export const {} = z.lazy(() => {});", #name, #schema)
                        }
                    }
                }
                _ => {
                    // make sure all fields followed an optional field are also optional
                    if let Some(f) = fields
                        .iter()
                        .skip_while(|f| !f.optional)
                        .find(|f| !f.optional)
                    {
                        abort!(f.ty.span(), "zod: non-default field follows default field")
                    }

                    let len = fields.len();
                    let fields = fields.into_iter().enumerate().map(|(i, f)| {
                        if i + 1 == len {
                            f.expand_schema()
                        } else {
                            let schema = f.expand_schema();
                            quote!(#concatcp!(#schema, ", "))
                        }
                    });

                    if has_generics {
                        quote! {
                            #formatcp!("export const {} = ({}) => z.lazy(() => z.tuple([{}]));", #name, #generic_params, #concatcp!(#(#fields),*))
                        }
                    } else {
                        quote! {
                            #formatcp!("export const {} = z.lazy(() => z.tuple([{}]));", #name, #concatcp!(#(#fields),*))
                        }
                    }
                }
            },

            (false, Style::Struct) => {
                let len = fields.len();
                let fields = fields.into_iter().enumerate().map(|(i, f)| {
                    if i + 1 == len {
                        f.expand_schema()
                    } else {
                        let f = f.expand_schema();
                        quote!(#concatcp!(#f, ", "))
                    }
                });

                let flat_fields = flat_fields.into_iter().map(|f| f.expand_schema());
                let name = &self.name;

                if has_generics {
                    quote! {
                        #formatcp!("export const {} = ({}) => z.lazy(() => z.object({{{}}})){};", #name, #generic_params, #concatcp!("", #(#fields),*), #concatcp!(#(#flat_fields),*))
                    }
                } else {
                    quote! {
                        #formatcp!("export const {} = z.lazy(() => z.object({{{}}})){};", #name, #concatcp!("", #(#fields),*), #concatcp!(#(#flat_fields),*))
                    }
                }
            }

            (false, Style::Unit) => unreachable!(),
        }
    }
}

struct StructField<'a> {
    name: Option<String>,
    ty: &'a syn::Type,
    optional: bool,
    transparent: bool,
    flatten: bool,
    generic_params: &'a [syn::Ident],
}

impl<'a> StructField<'a> {
    fn get_generic(&self) -> Option<String> {
        self.generic_params.iter().find_map(|i| match self.ty {
            syn::Type::Path(p) => {
                if p.path.segments.len() != 1 {
                    None
                } else {
                    let first = p.path.segments.first().unwrap();
                    if &first.ident == i {
                        Some(i.to_string())
                    } else {
                        None
                    }
                }
            }
            _ => None,
        })
    }

    fn expand_schema(&self) -> TokenStream {
        let maybe_optional = self.expand_optional_schema();
        let zod = get_zod();
        let formatcp = quote!(#zod::__private::const_format::formatcp);
        let concatcp = quote!(#zod::__private::const_format::concatcp);
        let span = self.ty.span();

        let ty_name = self
            .get_generic()
            .map(|name| quote!(#name))
            .unwrap_or_else(|| {
                let field_generic_params = match self.ty {
                    syn::Type::Path(p) => {
                        if let Some(x) = p.path.segments.last() {
                            match &x.arguments {
                                syn::PathArguments::AngleBracketed(args) => args
                                    .args
                                    .iter()
                                    .filter_map(|arg| match arg {
                                        syn::GenericArgument::Type(t) => {
                                            let ty = qualified_ty(t);
                                            Some(quote!(#formatcp!("{}.{},", #ty::AST.ns_name, #ty::AST.name)))
                                        }
                                        _ => None,
                                    })
                                    .collect::<Vec<_>>(),
                                _ => vec![],
                            }
                        } else {
                            vec![]
                        }
                    }
                    _ => vec![],
                };

                if field_generic_params.is_empty() {
                    let ty = qualified_ty(self.ty);
                    quote!(#formatcp!("{}.{}", #ty::AST.ns_name, #ty::AST.name))
                } else {
                    let ty = qualified_ty(self.ty);
                    quote!(#formatcp!("{}.{}({})", #ty::AST.ns_name, #ty::AST.name, #concatcp!(#(#field_generic_params),*)))
                }
            });

        match (self.flatten, &self.name, self.transparent) {
            (false, Some(name), false) => {
                quote_spanned! {span =>  #formatcp!("{}: {}{}", #name, #ty_name, #maybe_optional) }
            }
            (false, Some(_), true) => {
                quote_spanned! {span =>  #formatcp!("{}{}", #ty_name, #maybe_optional) }
            }
            (false, None, _) => {
                quote_spanned! { span => #formatcp!("{}{}", #ty_name, #maybe_optional) }
            }

            (true, Some(_), false) => {
                quote_spanned! {span =>  #formatcp!(".extend(z.lazy(() => {}{}))", #ty_name, #maybe_optional) }
            }
            (true, Some(_), true) => {
                quote_spanned! {span =>  #formatcp!(".extend(z.lazy(() => {}{}))", #ty_name, #maybe_optional) }
            }
            (true, None, _) => {
                // Newtype
                quote_spanned! { span => #formatcp!(".extend(z.lazy(() => {}{}))", #ty_name, #maybe_optional) }
            }
        }
    }

    fn expand_optional_schema(&self) -> TokenStream {
        if self.optional {
            quote!(".optional()")
        } else {
            quote!("")
        }
    }

    fn expand_type_defs(&self) -> TokenStream {
        let ty = qualified_ty(self.ty);
        let zod = get_zod();
        let formatcp = quote!(#zod::__private::const_format::formatcp);
        let concatcp = quote!(#zod::__private::const_format::concatcp);

        let ty_name = self
            .get_generic()
            .map(|name| quote!(#name))
            .unwrap_or_else(|| {
                let field_generic_params = match self.ty {
                    syn::Type::Path(p) => {
                        if let Some(x) = p.path.segments.last() {
                            match &x.arguments {
                                syn::PathArguments::AngleBracketed(args) => args
                                    .args
                                    .iter()
                                    .filter_map(|arg| match arg {
                                        syn::GenericArgument::Type(t) => {
                                            let ty = qualified_ty(t);
                                            Some(quote!(#formatcp!("{}.{},", #ty::AST.ns_name, #ty::AST.name)))
                                        }
                                        _ => None,
                                    })
                                    .collect::<Vec<_>>(),
                                _ => vec![],
                            }
                        } else {
                            vec![]
                        }
                    }
                    _ => vec![],
                };

                if field_generic_params.is_empty() {
                    let ty = qualified_ty(self.ty);
                    quote!(#formatcp!("{}.{}", #ty::AST.ns_name, #ty::AST.name))
                } else {
                    let ty = qualified_ty(self.ty);
                    quote!(#formatcp!("{}.{}<{}>", #ty::AST.ns_name, #ty::AST.name, #concatcp!(#(#field_generic_params),*)))
                }
            });

        match (self.flatten, &self.name, self.optional, self.transparent) {
            (false, Some(name), false, false) => {
                quote_spanned! {ty.span() =>  #formatcp!("{}: {}", #name, #ty_name) }
            }
            (false, None, false, false) => {
                // Newtype
                quote_spanned! {ty.span() => #ty_name}
            }
            (false, Some(name), true, false) => {
                quote_spanned! {ty.span() =>  #formatcp!("{}?: {} | undefined", #name, #ty_name) }
            }
            (false, None, true, false) => {
                // Newtype
                quote_spanned! {ty.span() => #formatcp!("{} | undefined", #ty_name)}
            }

            (false, _, false, true) => {
                // Newtype
                quote_spanned! {ty.span() => #ty_name }
            }
            (false, _, true, true) => {
                quote_spanned! {ty.span() =>  #formatcp!("{} | undefined", #ty_name) }
            }

            (true, Some(_), false, false) => {
                quote_spanned! {ty.span() =>  #formatcp!("{}", #ty_name) }
            }
            (true, None, false, false) => {
                // Newtype
                quote_spanned! {ty.span() => #ty_name }
            }

            (true, _, false, true) => {
                // Newtype
                quote_spanned! {ty.span() => #ty_name}
            }

            (true, _, true, _) => {
                abort!(
                    self.ty.span(),
                    "flatten and default may not be called on the same field"
                )
            }
        }
    }
}
