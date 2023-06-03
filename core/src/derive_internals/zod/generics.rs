use crate::utils::zod_core;
use quote::quote;
use std::collections::HashMap;
use std::collections::HashSet;
use syn::{
    parse_quote, visit,
    visit::Visit,
    visit_mut::{self, VisitMut},
};

use super::Derive;

pub(super) trait GenericsExt {
    fn idents(&self) -> Vec<&syn::Ident>;
    fn update_where_clause(&mut self, derive: Derive);
}

impl GenericsExt for syn::Generics {
    fn idents(&self) -> Vec<&syn::Ident> {
        self.params
            .iter()
            .map(|p| match p {
                syn::GenericParam::Lifetime(_) => todo!(),
                syn::GenericParam::Type(param) => &param.ident,
                syn::GenericParam::Const(_) => todo!(),
            })
            .collect()
    }

    fn update_where_clause(&mut self, derive: Derive) {
        if let Some(ref mut clause) = self.where_clause {
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
            let idents = self.idents();
            let predicates = idents
                .iter()
                .map(|ident| quote!(#ident: #zod_core::Type<#derive>));

            self.where_clause = Some(parse_quote!(where #(#predicates),*))
        };
    }
}

struct GenercicsReplace {
    generics: HashSet<syn::Ident>,
}

struct InlineVisitor {
    generics: HashMap<syn::Ident, syn::TypeParam>,
    value: bool,
}

impl Visit<'_> for InlineVisitor {
    fn visit_type(&mut self, node: &syn::Type) {
        match node {
            syn::Type::Path(p) => {
                if let Some(orig) = p.path.get_ident() {
                    if let Some(p) = self.generics.get(orig) {
                        if !p.bounds.is_empty() {
                            self.value = true;
                            return;
                        }
                    }
                }
            }
            _ => {}
        }
        visit::visit_type(self, node)
    }
}

impl VisitMut for GenercicsReplace {
    fn visit_type_mut(&mut self, node: &mut syn::Type) {
        match node {
            syn::Type::Path(p) => {
                if let Some(orig) = p.path.get_ident() {
                    if self.generics.get(orig).is_some() {
                        let ident = make_generic_struct_ident(orig);
                        *node = parse_quote!(#zod_core::GenericPlaceholder<#ident>);
                    }
                }
            }
            _ => {}
        }
        visit_mut::visit_type_mut(self, node)
    }
}

pub(crate) fn make_generic_struct_ident(ident: &syn::Ident) -> syn::Ident {
    quote::format_ident!("__GENERIC_{ident}")
}

pub(crate) fn replace_generics(ty: &mut syn::Type, generics: &syn::Generics) {
    let mut visitor = GenercicsReplace {
        generics: generics
            .params
            .iter()
            .filter_map(|p| match p {
                syn::GenericParam::Lifetime(_) => todo!(),
                syn::GenericParam::Type(t) => Some(t.ident.clone()),
                syn::GenericParam::Const(_) => todo!(),
            })
            .collect(),
    };
    visitor.visit_type_mut(ty);
}

pub(crate) fn needs_inline(ty: &syn::Type, generics: &syn::Generics) -> bool {
    match ty {
        syn::Type::Path(p) => {
            if p.path.get_ident().is_some() {
                return false;
            }
        }
        _ => {}
    }
    let generics = generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Lifetime(_) => todo!(),
            syn::GenericParam::Type(t) => Some((t.ident.clone(), t.clone())),
            syn::GenericParam::Const(_) => todo!(),
        })
        .collect();

    let mut visitor = InlineVisitor {
        generics,
        value: false,
    };
    visitor.visit_type(ty);
    visitor.value
}

#[cfg(test)]
mod test {

    use super::*;
    use darling::ToTokens;
    use pretty_assertions::assert_eq;
    use syn::parse_quote;

    #[test]
    fn ok() {
        let mut input: syn::Type = parse_quote!(Test<A, LONG, C<D, NotUsed>>);
        let a = make_generic_struct_ident(&parse_quote!(A));
        let long = make_generic_struct_ident(&parse_quote!(LONG));
        let d = make_generic_struct_ident(&parse_quote!(D));

        let expected: syn::Type = parse_quote!(Test<#zod_core::GenericPlaceholder<#a>, #zod_core::GenericPlaceholder<#long>, C<#zod_core::GenericPlaceholder<#d>, NotUsed>>);
        let generics = parse_quote!(<A, LONG, D>);
        replace_generics(&mut input, &generics);

        assert_eq!(
            input.into_token_stream().to_string(),
            expected.into_token_stream().to_string()
        );
    }

    #[test]
    fn needs_inline_ok() {
        let input: syn::Type = parse_quote!(Test<A, LONG, C<D, NotUsed>>);
        let input2: syn::Type = parse_quote!(A);
        assert_eq!(needs_inline(&input, &parse_quote!(<A, LONG, D>)), false);
        assert_eq!(
            needs_inline(&input, &parse_quote!(<A: MyTrait, LONG, D>)),
            true
        );
        assert_eq!(
            needs_inline(&input2, &parse_quote!(<A: MyTrait, LONG, D>)),
            false
        );
    }
}
