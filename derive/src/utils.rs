use proc_macro2::Span;
use syn::{Ident, Path, Type, TypePath};

use {::core::ops::Not, ::syn::visit::Visit};

fn get_crate_name() -> String {
    proc_macro_crate::crate_name("zod")
        .map(|found_crate| match found_crate {
            proc_macro_crate::FoundCrate::Itself => String::from("zod"),
            proc_macro_crate::FoundCrate::Name(name) => name,
        })
        .unwrap_or_else(|_| String::from("zod"))
}

pub(crate) fn get_zod() -> Path {
    let name = get_crate_name();
    let ident = Ident::new(&name, Span::call_site());
    syn::parse_quote!(::#ident)
}

/// visit all idents and remove them from the unseen list.
/// The unseen list is the list of the possible
pub(crate) fn generics_of_ty<'generics>(
    generics: &'generics syn::Generics,
    ty: &'_ Type,
) -> Vec<&'generics Ident> {
    struct IdentVisitor<'a> {
        unseen_idents: Vec<&'a Ident>,
    }

    impl<'i> Visit<'i> for IdentVisitor<'_> {
        fn visit_ident(&mut self, ident: &'i Ident) {
            self.unseen_idents.retain(|&generic| ident != generic);
        }
    }

    let generic_idents = || generics.type_params().map(|it| &it.ident);

    let mut visitor = IdentVisitor {
        unseen_idents: generic_idents().collect(),
    };

    visitor.visit_type(ty);

    generic_idents()
        .filter(|ty| visitor.unseen_idents.contains(ty).not())
        .collect()
}

/// Check if item can be exported
///
/// 1. It does not have generics
/// 2. All field values match a generic param
///
/// # Example
/// ```
/// // EXPORT
/// struct MyType1<T> {
///   field: T
/// }
///
/// // INLINE
/// struct MyType2<T1, T2> {
///   field1: T1,
///   field2: Result<T2, String>
/// }
/// ```
///
pub fn is_export(
    fields: impl IntoIterator<Item = crate::field::Field>,
    generics: &syn::Generics,
) -> bool {
    fields.into_iter().all(|f| {
        // the type does not have generics buy may be a generic param itself, which is ok
        if let Type::Path(TypePath { qself: None, path }) = &f.ty {
            if path.get_ident().is_some() {
                return true;
            }
        }

        generics_of_ty(generics, &f.ty).is_empty()
    })
}

#[cfg(test)]
mod test {
    use syn::parse_quote;

    use crate::field::Field;

    use super::*;

    #[test]
    fn generics_for_ty_path_ok() {
        let generics = parse_quote!(<A, B, C>);
        let ty = Type::Path(parse_quote!(HashSet<String, Vec<A>>));
        let res = generics_of_ty(&generics, &ty);
        let ident: Ident = parse_quote!(A);
        assert_eq!(res, vec![&ident]);
    }

    #[test]
    fn generics_for_ty_array_ok() {
        let generics = parse_quote!(<A, B, C>);
        let ty = Type::Array(parse_quote!([A; 5]));
        let res = generics_of_ty(&generics, &ty);
        let ident: Ident = parse_quote!(A);
        assert_eq!(res, vec![&ident]);
    }

    #[test]
    fn generics_for_ty_array_const_ok() {
        let generics = parse_quote!(<A, B, C>);
        let ty = Type::Array(parse_quote!([usize; B]));
        let res = generics_of_ty(&generics, &ty);
        let ident: Ident = parse_quote!(B);
        assert_eq!(res, vec![&ident]);
    }

    #[test]
    fn generics_for_ty_slice_ok() {
        let generics = parse_quote!(<A, B, C>);
        let ty = Type::Slice(parse_quote!([A]));
        let res = generics_of_ty(&generics, &ty);
        let ident: Ident = parse_quote!(A);
        assert_eq!(res, vec![&ident]);
    }

    #[test]
    fn is_export_no_generics_no_fields_ok() {
        let generics = Default::default();
        assert!(is_export(Vec::new(), &generics));
    }

    #[test]
    fn is_export_no_generics_fields_ok() {
        let fields = vec![
            Field {
                ty: parse_quote!(Vec<T>),
                config: Default::default(),
            },
            Field {
                ty: parse_quote!(Option<bool>),
                config: Default::default(),
            },
        ];

        let generics = Default::default();
        assert!(is_export(fields, &generics));
    }

    #[test]
    fn is_export_generics_and_fields_ok() {
        let fields = vec![
            Field {
                ty: parse_quote!(T),
                config: Default::default(),
            },
            Field {
                ty: parse_quote!(Option<bool>),
                config: Default::default(),
            },
        ];
        let generics = parse_quote!(<T>);
        assert!(is_export(fields, &generics));
    }

    #[test]
    fn is_not_export_generics_and_fields_ok2() {
        let generics = parse_quote!(<T>);
        let fields = vec![Field {
            ty: parse_quote!(Result<String, T>),
            config: Default::default(),
        }];

        assert!(!is_export(fields, &generics))
    }

    #[test]
    fn is_not_export_generics_and_fields_ok3() {
        let generics = parse_quote!(<T1, T2>);
        let fields = vec![
            Field {
                ty: parse_quote!(T1),
                config: Default::default(),
            },
            Field {
                ty: parse_quote!(T2),
                config: Default::default(),
            },
            Field {
                ty: parse_quote!(HashMap<T2, bool>),
                config: Default::default(),
            },
        ];

        assert!(!is_export(fields, &generics))
    }
}
