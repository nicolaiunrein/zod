use crate::utils::zod_core;
use std::collections::HashSet;

use syn::parse_quote;

use syn::visit_mut::{self, VisitMut};

struct GenercicsReplace {
    generics: HashSet<syn::Ident>,
}

impl VisitMut for GenercicsReplace {
    fn visit_type_mut(&mut self, node: &mut syn::Type) {
        match node {
            syn::Type::Path(p) => {
                if let Some(orig) = p.path.get_ident() {
                    if self.generics.get(orig).is_some() {
                        let name = orig.to_string();
                        let chars = name.chars();
                        *node = make_generic(chars);
                    }
                }
            }
            _ => {}
        }
        visit_mut::visit_type_mut(self, node)
    }
}
fn make_generic(mut chars: impl Iterator<Item = char>) -> syn::Type {
    match chars.next() {
        Some(c) => {
            let inner = make_generic(chars);
            parse_quote!(#zod_core::const_str::ConstStr<#c, #inner>)
        }
        None => parse_quote!(#zod_core::const_str::End),
    }
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

#[cfg(test)]
mod test {

    use super::*;
    use darling::ToTokens;
    use pretty_assertions::assert_eq;
    use syn::parse_quote;

    #[test]
    fn ok() {
        let mut input: syn::Type = parse_quote!(Test<A, LONG, C<D, NotUsed>>);
        let a = make_generic("A".chars());
        let long = make_generic("LONG".chars());
        let d = make_generic("D".chars());

        let expected: syn::Type = parse_quote!(Test<#a, #long, C<#d, NotUsed>>);
        let generics = parse_quote!(<A, LONG, D>);
        replace_generics(&mut input, &generics);

        assert_eq!(
            input.into_token_stream().to_string(),
            expected.into_token_stream().to_string()
        );
    }
}
