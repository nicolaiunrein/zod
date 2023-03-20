use std::fmt::Display;

use super::{Delimited, FormatTypescript, FormatZod, MaybeFlatField, QualifiedType, StructFields};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Struct {
    pub ty: QualifiedType,
    pub fields: StructFields,
}

impl FormatZod for Struct {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut prefix = || {
            f.write_str("const ")?;
            f.write_str(self.ty.ident)?;
            f.write_str(" = ")?;
            if !self.ty.generics.is_empty() {
                f.write_str("(")?;
                let list = self
                    .ty
                    .generics
                    .iter()
                    .map(|g| format!("{}: z.ZodTypeAny", g.to_zod_string()))
                    .collect::<Vec<_>>();

                Delimited(list.as_slice(), ", ").fmt(f)?;
                f.write_str(")")?;
                f.write_str(" => ")?;
            }
            Ok(())
        };

        match self.fields {
            StructFields::Named(fields) => {
                prefix()?;
                f.write_str("z.lazy(() => z.object({")?;

                let (inner_fields, flat_fields) = MaybeFlatField::partition(&fields);

                Delimited(inner_fields.as_slice(), ", ").fmt_zod(f)?;

                f.write_str("}))")?;

                for flat_field in flat_fields {
                    flat_field.fmt_zod(f)?;
                }

                f.write_str(";")?;
            }
            StructFields::Tuple(fields) => {
                if fields.len() == 1 {
                    let field = fields.first().expect("one field");

                    Self {
                        ty: self.ty,
                        fields: StructFields::Transparent {
                            optional: field.optional,
                            value: field.value.clone(),
                        },
                    }
                    .fmt_zod(f)?;
                } else {
                    prefix()?;
                    f.write_str("z.lazy(() => z.tuple([")?;

                    Delimited(fields, ", ").fmt_zod(f)?;
                    f.write_str("]));")?;
                }
            }
            StructFields::Transparent {
                ref value,
                optional,
            } => {
                prefix()?;
                f.write_str("z.lazy(() => ")?;
                value.fmt_zod(f)?;
                if optional {
                    f.write_str(".optional()")?;
                }
                f.write_str(")")?;
                f.write_str(";")?;
            }
        }
        Ok(())
    }
}

impl FormatTypescript for Struct {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.fields {
            StructFields::Named(fields) => {
                let (inner_fields, flat_fields) = MaybeFlatField::partition(fields);
                f.write_str("interface ")?;
                self.ty.as_name().fmt_ts(f)?;

                if !flat_fields.is_empty() {
                    f.write_str(" extends ")?;
                }

                Delimited(flat_fields.as_slice(), ", ").fmt_ts(f)?;

                f.write_str(" { ")?;

                Delimited(inner_fields.as_slice(), ", ").fmt_ts(f)?;

                f.write_str(" }")?;
            }
            StructFields::Tuple(fields) => match fields.len() {
                1 => {
                    let field = fields.first().expect("one field");
                    Self {
                        ty: self.ty,
                        fields: StructFields::Transparent {
                            optional: field.optional,
                            value: field.value.clone(),
                        },
                    }
                    .fmt_ts(f)?;
                }
                _ => {
                    f.write_str("type ")?;
                    self.ty.as_name().fmt_ts(f)?;
                    f.write_str(" = [")?;
                    Delimited(fields, ", ").fmt_ts(f)?;
                    f.write_str("]")?;
                    f.write_str(";")?;
                }
            },
            StructFields::Transparent {
                ref value,
                optional,
            } => {
                f.write_str("type ")?;
                self.ty.as_name().fmt_ts(f)?;
                f.write_str(" = ")?;
                value.fmt_ts(f)?;
                if optional {
                    f.write_str(" | undefined")?;
                }

                f.write_str(";")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::ast::{
        FieldValue, FlatField, FormatTypescript, Generic, NamedField, QualifiedType, TupleField,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn zod_tuple_struct() {
        const STRUCT: Struct = Struct {
            ty: QualifiedType {
                ns: "Ns",
                ident: "test",
                generics: &[],
            },
            fields: StructFields::Tuple(&[]),
        };

        assert_eq!(
            STRUCT.to_zod_string(),
            format!(
                "const {} = z.lazy(() => z.tuple([]));",
                STRUCT.ty.as_name().to_zod_string()
            )
        );

        assert_eq!(
            STRUCT.to_ts_string(),
            format!("type {} = [];", STRUCT.ty.as_name().to_ts_string())
        );
    }

    #[test]
    fn zod_tuple_struct_with_generics() {
        const STRUCT: Struct = Struct {
            ty: QualifiedType {
                ns: "Ns",
                ident: "test",
                generics: &[Generic::new_for::<()>("A"), Generic::new_for::<()>("B")],
            },
            fields: StructFields::Tuple(&[]),
        };

        assert_eq!(
            STRUCT.to_zod_string(),
            "const test = (A: z.ZodTypeAny, B: z.ZodTypeAny) => z.lazy(() => z.tuple([]));",
        );

        assert_eq!(
            STRUCT.to_ts_string(),
            format!("type {} = [];", STRUCT.ty.as_name().to_ts_string(),)
        );
    }

    #[test]
    fn zod_tuple_struct_with_generics_and_fields() {
        const FIELDS: &[TupleField] = &[
            TupleField {
                optional: false,
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Ns",
                    ident: "a",
                    generics: &[Generic::new_for::<()>("A")],
                }),
            },
            TupleField {
                optional: false,
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Ns",
                    ident: "b",
                    generics: &[Generic::new_for::<()>("B")],
                }),
            },
            TupleField {
                optional: false,
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Ns",
                    ident: "c",
                    generics: &[],
                }),
            },
            TupleField {
                optional: true,
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Ns",
                    ident: "d",
                    generics: &[],
                }),
            },
        ];

        const STRUCT: Struct = Struct {
            ty: QualifiedType {
                ns: "Ns",
                ident: "test",
                generics: &[Generic::new_for::<()>("A"), Generic::new_for::<()>("B")],
            },
            fields: StructFields::Tuple(FIELDS),
        };

        assert_eq!(
            STRUCT.to_zod_string(),
            "const test = (A: z.ZodTypeAny, B: z.ZodTypeAny) => z.lazy(() => z.tuple([Ns.a(A), Ns.b(B), Ns.c, Ns.d.optional()]));",
        );

        assert_eq!(
            STRUCT.to_ts_string(),
            format!(
                "type {} = [Ns.a<A>, Ns.b<B>, Ns.c, Ns.d | undefined];",
                STRUCT.ty.as_name().to_ts_string(),
            )
        );
    }

    #[test]
    fn zod_named_struct() {
        const STRUCT: Struct = Struct {
            ty: QualifiedType {
                ns: "Ns",
                ident: "test",
                generics: &[],
            },
            fields: StructFields::Named(&[]),
        };

        assert_eq!(
            STRUCT.to_zod_string(),
            format!(
                "const {} = z.lazy(() => z.object({{}}));",
                STRUCT.ty.as_name().to_zod_string()
            )
        );
    }

    #[test]
    fn zod_named_struct_with_generics_and_fields() {
        const FIELDS: &[MaybeFlatField] = &[
            MaybeFlatField::Named(NamedField {
                optional: false,
                name: "hallo_a",
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Ns",
                    ident: "a",
                    generics: &[Generic::new_for::<()>("A")],
                }),
            }),
            MaybeFlatField::Named(NamedField {
                optional: false,
                name: "hallo_b",
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Ns",
                    ident: "b",
                    generics: &[Generic::new_for::<()>("B")],
                }),
            }),
            MaybeFlatField::Named(NamedField {
                optional: false,
                name: "hallo_c",
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Ns",
                    ident: "c",
                    generics: &[],
                }),
            }),
            MaybeFlatField::Named(NamedField {
                optional: true,
                name: "hallo_d",
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Ns",
                    ident: "d",
                    generics: &[],
                }),
            }),
            MaybeFlatField::Flat(FlatField {
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Ns",
                    ident: "e",
                    generics: &[],
                }),
            }),
        ];

        const STRUCT: Struct = Struct {
            ty: QualifiedType {
                ns: "Ns",
                ident: "test",
                generics: &[Generic::new_for::<()>("A"), Generic::new_for::<()>("B")],
            },
            fields: StructFields::Named(FIELDS),
        };

        assert_eq!(
            STRUCT.to_zod_string(),
            "const test = (A: z.ZodTypeAny, B: z.ZodTypeAny) => z.lazy(() => z.object({hallo_a: Ns.a(A), hallo_b: Ns.b(B), hallo_c: Ns.c, hallo_d: Ns.d.optional()})).extend(z.lazy(() => Ns.e));",
        );

        assert_eq!(
            STRUCT.to_ts_string(),
            "interface test<A, B> extends Ns.e { hallo_a: Ns.a<A>, hallo_b: Ns.b<B>, hallo_c: Ns.c, hallo_d?: Ns.d | undefined }"
        );
    }

    #[test]
    fn transparent_field() {
        const STRUCT: Struct = Struct {
            ty: QualifiedType {
                ns: "Ns",
                ident: "test",
                generics: &[],
            },
            fields: StructFields::Transparent {
                optional: false,
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Ns",
                    ident: "inner",
                    generics: &[],
                }),
            },
        };

        assert_eq!(
            STRUCT.to_zod_string(),
            "const test = z.lazy(() => Ns.inner);"
        );
        assert_eq!(STRUCT.to_ts_string(), "type test = Ns.inner;");
    }

    #[test]
    fn transparent_field_generics() {
        const STRUCT: Struct = Struct {
            ty: QualifiedType {
                ns: "Ns",
                ident: "test",
                generics: &[Generic::new_for::<()>("A"), Generic::new_for::<()>("B")],
            },
            fields: StructFields::Transparent {
                optional: false,
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Ns",
                    ident: "inner",
                    generics: &[Generic::new_for::<()>("A"), Generic::new_for::<()>("B")],
                }),
            },
        };

        assert_eq!(
            STRUCT.to_zod_string(),
            "const test = (A: z.ZodTypeAny, B: z.ZodTypeAny) => z.lazy(() => Ns.inner(A, B));"
        );

        assert_eq!(STRUCT.to_ts_string(), "type test<A, B> = Ns.inner<A, B>;")
    }

    #[test]
    fn tuple_with_one_required_field_gets_flattened() {
        const STRUCT: Struct = Struct {
            ty: QualifiedType {
                ns: "Ns",
                ident: "test",
                generics: &[],
            },
            fields: StructFields::Tuple(&[TupleField {
                optional: false,
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Other",
                    ident: "other",
                    generics: &[],
                }),
            }]),
        };

        assert_eq!(
            STRUCT.to_zod_string(),
            "const test = z.lazy(() => Other.other);"
        );

        assert_eq!(STRUCT.to_ts_string(), "type test = Other.other;");
    }

    #[test]
    fn tuple_with_one_optional_field_gets_flattened() {
        const STRUCT: Struct = Struct {
            ty: QualifiedType {
                ns: "Ns",
                ident: "test",
                generics: &[],
            },
            fields: StructFields::Tuple(&[TupleField {
                optional: true,
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Other",
                    ident: "other",
                    generics: &[],
                }),
            }]),
        };

        assert_eq!(
            STRUCT.to_zod_string(),
            "const test = z.lazy(() => Other.other.optional());"
        );

        assert_eq!(
            STRUCT.to_ts_string(),
            "type test = Other.other | undefined;"
        );
    }

    #[test]
    #[ignore]
    fn generic_inline() {
        const STRUCT: Struct = Struct {
            ty: QualifiedType {
                ns: "Ns",
                ident: "test",
                generics: &[],
            },
            fields: StructFields::Tuple(&[TupleField {
                optional: false,
                value: FieldValue::Qualified(QualifiedType {
                    ns: "Other",
                    ident: "Generic",
                    generics: &[
                        Generic::new_for::<String>("A"),
                        Generic::new_for::<usize>("B"),
                    ],
                }),
            }]),
        };

        assert_eq!(
            STRUCT.to_zod_string(),
            "const test = z.lazy(() => Other.Generic(Rs.String, Rs.Usize));"
        );

        assert_eq!(
            STRUCT.to_ts_string(),
            "type test = Other.Generic<Rs.String, Rs.Usize>;"
        );
    }
}
