use crate::ZodType;

use super::{FormatTypescript, FormatZod, TypeDef};

#[derive(Clone, Copy, Debug)]
pub struct GenericMap(&'static phf::Map<u64, &'static str>);

impl PartialEq for GenericMap {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        self.0
            .into_iter()
            .zip(other.0.into_iter())
            .all(|(a, b)| a == b)
    }
}

impl Eq for GenericMap {
    fn assert_receiver_is_total_eq(&self) {}
}

impl std::hash::Hash for GenericMap {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for entry in self.0.into_iter() {
            entry.hash(state)
        }
    }
}

impl GenericMap {
    pub const fn empty() -> Self {
        Self(&phf::phf_map! {})
    }

    pub const fn new(map: &'static phf::Map<u64, &'static str>) -> Self {
        Self(map)
    }
}

impl std::ops::Deref for GenericMap {
    type Target = &'static phf::Map<u64, &'static str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StructFields {
    Named(&'static [MaybeFlatField]),
    Tuple(&'static [TupleField]),
    Transparent { value: FieldValue, optional: bool },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FieldValue {
    def: TypeDef,
    generics: GenericMap,
}

impl FieldValue {
    pub const fn new_for<T: ZodType>(map: &'static phf::Map<u64, &'static str>) -> Self {
        Self {
            def: T::AST.def.ty(),
            generics: GenericMap(map),
        }
    }

    #[cfg(debug_assertions)]
    pub const fn empty(def: TypeDef) -> Self {
        Self {
            def,
            generics: GenericMap::empty(),
        }
    }

    #[cfg(debug_assertions)]
    pub const fn new(def: TypeDef, map: &'static phf::Map<u64, &'static str>) -> Self {
        Self {
            def,
            generics: GenericMap::new(map),
        }
    }
}

impl FormatZod for FieldValue {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.def.as_arg(self.generics).fmt_zod(f)
    }
}

impl FormatTypescript for FieldValue {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.def.as_arg(self.generics).fmt_ts(f)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MaybeFlatField {
    Flat(FlatField),
    Named(NamedField),
}

impl MaybeFlatField {
    pub fn partition(fields: &[Self]) -> (Vec<NamedField>, Vec<FlatField>) {
        let mut inner = Vec::new();
        let mut flat = Vec::new();

        for field in fields.into_iter() {
            match field {
                Self::Flat(f) => flat.push(f.clone()),
                Self::Named(f) => inner.push(f.clone()),
            }
        }

        (inner, flat)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FlatField {
    // TODO: find a way to express flat optional fields in typescript with interfaces
    // see: https://github.com/nicolaiunrein/zod/issues/3
    pub value: FieldValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TupleField {
    pub optional: bool,
    pub value: FieldValue,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NamedField {
    pub optional: bool,
    pub name: &'static str,
    pub value: FieldValue,
}

impl MaybeFlatField {
    pub const fn new_flat(value: FieldValue) -> Self {
        Self::Flat(FlatField { value })
    }
}

impl FormatZod for NamedField {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)?;
        f.write_str(": ")?;
        self.value.fmt_zod(f)?;
        if self.optional {
            f.write_str(".optional()")?;
        }
        Ok(())
    }
}

impl FormatTypescript for NamedField {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)?;
        if self.optional {
            f.write_str("?")?;
        }
        f.write_str(": ")?;
        self.value.fmt_ts(f)?;
        if self.optional {
            f.write_str(" | undefined")?;
        }
        Ok(())
    }
}

impl FormatZod for TupleField {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt_zod(f)?;
        if self.optional {
            f.write_str(".optional()")?;
        }
        Ok(())
    }
}

impl FormatTypescript for TupleField {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt_ts(f)?;
        if self.optional {
            f.write_str(" | undefined")?;
        }
        Ok(())
    }
}

impl FormatZod for FlatField {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(".extend(z.lazy(() => ")?;
        self.value.fmt_zod(f)?;
        f.write_str("))")?;
        Ok(())
    }
}

impl FormatTypescript for FlatField {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt_zod(f)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn zod_inner_tuple_struct_field() {
        assert_eq!(
            TupleField {
                optional: false,
                value: FieldValue::empty(TypeDef {
                    ns: "Ns",
                    ident: "myValue",
                    generics: Default::default()
                },)
            }
            .to_zod_string(),
            "Ns.myValue"
        );
    }

    #[test]
    fn zod_inner_tuple_struct_field_optional() {
        let field = TupleField {
            optional: true,
            value: FieldValue::empty(TypeDef {
                ns: "Ns",
                ident: "myValue",
                generics: Default::default(),
            }),
        };
        assert_eq!(field.to_zod_string(), "Ns.myValue.optional()");
        assert_eq!(field.to_ts_string(), "Ns.myValue | undefined");
    }

    #[test]
    fn zod_named_struct_field() {
        let field = NamedField {
            optional: false,
            name: "my_value",
            value: FieldValue::empty(TypeDef {
                ns: "Ns",
                ident: "myValue",
                generics: Default::default(),
            }),
        };
        assert_eq!(field.to_zod_string(), "my_value: Ns.myValue");
        assert_eq!(field.to_ts_string(), "my_value: Ns.myValue");
    }

    #[test]
    fn zod_named_struct_field_optional() {
        let field = NamedField {
            optional: true,
            name: "my_value",
            value: FieldValue::empty(TypeDef {
                ns: "Ns",
                ident: "myValue",
                generics: Default::default(),
            }),
        };
        assert_eq!(field.to_zod_string(), "my_value: Ns.myValue.optional()");
        assert_eq!(field.to_ts_string(), "my_value?: Ns.myValue | undefined");
    }

    #[test]
    fn flattened_field() {
        let field = FlatField {
            value: FieldValue::empty(TypeDef {
                ns: "Ns",
                ident: "myValue",
                generics: Default::default(),
            }),
        };
        assert_eq!(field.to_zod_string(), ".extend(z.lazy(() => Ns.myValue))");
        assert_eq!(field.to_ts_string(), "Ns.myValue");
    }

    #[test]
    fn ok_with_empty_generics() {
        let res = FieldValue::new_for::<HashMap<String, usize>>(&GenericMap::empty());

        assert_eq!(res.to_zod_string(), "Rs.HashMap(Rs.String, Rs.Usize)");
        assert_eq!(res.to_ts_string(), "Rs.HashMap<Rs.String, Rs.Usize>");
    }
    #[test]
    fn ok_with_partial_generics() {
        let res = FieldValue::new_for::<HashMap<String, usize>>(&GenericMap::new(
            &phf::phf_map! { 1_u64 => "T"},
        ));

        assert_eq!(res.to_zod_string(), "Rs.HashMap(Rs.String, T)");
        assert_eq!(res.to_ts_string(), "Rs.HashMap<Rs.String, T>");
    }
}
