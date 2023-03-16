use std::fmt::Display;

use super::{
    AnyNamedField, AnyTupleField, Delimited, FormatTypescript, FormatZod, StructFields, Type,
};

#[derive(Clone, Copy, Debug)]
pub struct Struct {
    pub ns: &'static str,
    pub ty: Type,
    pub fields: StructFields,
}

impl FormatZod for Struct {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

        match self.fields {
            StructFields::Named(fields) => {
                f.write_str("z.lazy(() => z.object({")?;

                let (inner_fields, flat_fields) = AnyNamedField::partition(fields);

                Delimited(inner_fields.as_slice(), ", ").fmt_zod(f)?;

                f.write_str("}))")?;

                for flat_field in flat_fields {
                    flat_field.fmt_zod(f)?;
                }

                f.write_str(";")?;
            }
            StructFields::Tuple(fields) => {
                f.write_str("z.lazy(() => z.tuple([")?;
                let inner_fields = fields
                    .into_iter()
                    .filter_map(|f| match f {
                        AnyTupleField::Inner(field) => Some(field),
                        _ => None,
                    })
                    .copied()
                    .collect::<Vec<_>>();

                Delimited(inner_fields.as_slice(), ", ").fmt_zod(f)?;
                f.write_str("]));")?;
            }
            StructFields::Transparent { value } => {
                f.write_str("z.lazy(() => ")?;
                value.fmt_zod(f)?;
                f.write_str(")")?;
            }
        }
        Ok(())
    }
}

impl FormatTypescript for Struct {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.fields {
            StructFields::Named(fields) => {
                let (inner_fields, flat_fields) = AnyNamedField::partition(fields);
                f.write_str("interface ")?;
                self.ty.fmt_ts(f)?;
                f.write_str(" {")?;

                Delimited(inner_fields.as_slice(), ", ").fmt_ts(f)?;

                f.write_str("}")?;

                for field in flat_fields {
                    f.write_str(" & ")?;
                    field.fmt_ts(f)?;
                }
            }
            StructFields::Tuple(fields) => {
                let (inner, flat) = AnyTupleField::partition(fields);
                if inner.is_empty() {
                    f.write_str("type ")?;
                    self.ty.fmt_ts(f)?;
                    f.write_str(" = []")?;
                    for ext in flat {
                        f.write_str("& ")?;
                        ext.fmt_ts(f)?;
                    }
                    f.write_str(";")?;
                } else {
                    f.write_str("type ")?;
                    self.ty.fmt_ts(f)?;
                    f.write_str(" = [")?;
                    Delimited(inner.as_slice(), ", ").fmt_ts(f)?;
                    f.write_str("]")?;
                    for ext in flat {
                        f.write_str("& ")?;
                        ext.fmt_ts(f)?;
                    }
                    f.write_str(";")?;
                }
            }
            StructFields::Transparent { value } => {
                f.write_str("type ")?;
                self.ty.fmt_ts(f)?;
                f.write_str(" = ")?;
                value.fmt_ts(f)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::formatter::{
        FlatField, FormatTypescript, Generic, NamedField, QualifiedType, TupleField,
    };

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn zod_tuple_struct() {
        let def = Struct {
            ns: "Ns",
            ty: Type {
                ident: "test",
                generics: Default::default(),
            },
            fields: StructFields::Tuple(&[]),
        };

        assert_eq!(
            def.to_zod_string(),
            format!(
                "const {} = z.lazy(() => z.tuple([]));",
                def.ty.to_zod_string()
            )
        );

        assert_eq!(
            def.to_ts_string(),
            format!("type {} = [];", def.ty.to_ts_string())
        );
    }

    #[test]
    fn zod_tuple_struct_with_generics() {
        let def = Struct {
            ns: "Ns",
            ty: Type {
                ident: "test",
                generics: &[Generic::Type { ident: "A" }, Generic::Type { ident: "B" }],
            },
            fields: StructFields::Tuple(&[]),
        };

        assert_eq!(
            def.to_zod_string(),
            "const test = (A: z.ZodTypeAny, B: z.ZodTypeAny) => z.lazy(() => z.tuple([]));",
        );

        assert_eq!(
            def.to_ts_string(),
            format!("type {} = [];", def.ty.to_ts_string(),)
        );
    }

    #[test]
    fn zod_tuple_struct_with_generics_and_fields() {
        let fields = &[
            AnyTupleField::Inner(TupleField {
                optional: false,
                value: QualifiedType {
                    ns: "Ns",
                    ident: "a",
                    generics: &[Generic::Type { ident: "A" }],
                },
            }),
            AnyTupleField::Inner(TupleField {
                optional: false,
                value: QualifiedType {
                    ns: "Ns",
                    ident: "b",
                    generics: &[Generic::Type { ident: "B" }],
                },
            }),
            AnyTupleField::Inner(TupleField {
                optional: false,
                value: QualifiedType {
                    ns: "Ns",
                    ident: "c",
                    generics: &[],
                },
            }),
            AnyTupleField::Inner(TupleField {
                optional: true,
                value: QualifiedType {
                    ns: "Ns",
                    ident: "d",
                    generics: &[],
                },
            }),
        ];

        let def = Struct {
            ns: "Ns",
            ty: Type {
                ident: "test",
                generics: &[Generic::Type { ident: "A" }, Generic::Type { ident: "B" }],
            },
            fields: StructFields::Tuple(fields),
        };

        assert_eq!(
            def.to_zod_string(),
            "const test = (A: z.ZodTypeAny, B: z.ZodTypeAny) => z.lazy(() => z.tuple([Ns.a(A), Ns.b(B), Ns.c, Ns.d.optional()]));",
        );

        assert_eq!(
            def.to_ts_string(),
            format!(
                "type {} = [Ns.a<A>, Ns.b<B>, Ns.c, Ns.d | undefined];",
                def.ty.to_ts_string(),
            )
        );
    }

    #[test]
    fn zod_named_struct() {
        let def = Struct {
            ns: "Ns",
            ty: Type {
                ident: "test",
                generics: Default::default(),
            },
            fields: StructFields::Named(&[]),
        };

        assert_eq!(
            def.to_zod_string(),
            format!(
                "const {} = z.lazy(() => z.object({{}}));",
                def.ty.to_zod_string()
            )
        );
    }

    #[test]
    fn zod_named_struct_with_generics_and_fields() {
        let fields = &[
            AnyNamedField::Inner(NamedField {
                optional: false,
                name: "hallo_a",
                value: QualifiedType {
                    ns: "Ns",
                    ident: "a",
                    generics: &[Generic::Type { ident: "A" }],
                },
            }),
            AnyNamedField::Inner(NamedField {
                optional: false,
                name: "hallo_b",
                value: QualifiedType {
                    ns: "Ns",
                    ident: "b",
                    generics: &[Generic::Type { ident: "B" }],
                },
            }),
            AnyNamedField::Inner(NamedField {
                optional: false,
                name: "hallo_c",
                value: QualifiedType {
                    ns: "Ns",
                    ident: "c",
                    generics: &[],
                },
            }),
            AnyNamedField::Inner(NamedField {
                optional: true,
                name: "hallo_d",
                value: QualifiedType {
                    ns: "Ns",
                    ident: "d",
                    generics: &[],
                },
            }),
            AnyNamedField::Flat(FlatField {
                value: QualifiedType {
                    ns: "Ns",
                    ident: "e",
                    generics: &[],
                },
            }),
        ];

        let def = Struct {
            ns: "Ns",
            ty: Type {
                ident: "test",
                generics: &[Generic::Type { ident: "A" }, Generic::Type { ident: "B" }],
            },
            fields: StructFields::Named(fields),
        };

        assert_eq!(
            def.to_zod_string(),
            "const test = (A: z.ZodTypeAny, B: z.ZodTypeAny) => z.lazy(() => z.object({hallo_a: Ns.a(A), hallo_b: Ns.b(B), hallo_c: Ns.c, hallo_d: Ns.d.optional()})).extend(Ns.e);",
        );

        assert_eq!(
            def.to_ts_string(),
            "interface test<A, B> {hallo_a: Ns.a<A>, hallo_b: Ns.b<B>, hallo_c: Ns.c, hallo_d?: Ns.d | undefined} & Ns.e"
        );
    }

    #[test]
    fn transparent_field() {
        let def = Struct {
            ns: "Ns",
            ty: Type {
                ident: "test",
                generics: &[],
            },
            fields: StructFields::Transparent {
                value: QualifiedType {
                    ns: "Ns",
                    ident: "inner",
                    generics: &[],
                },
            },
        };

        assert_eq!(def.to_zod_string(), "const test = z.lazy(() => Ns.inner)");
        assert_eq!(def.to_ts_string(), "type test = Ns.inner");
    }

    #[test]
    fn transparent_field_generics() {
        let def = Struct {
            ns: "Ns",
            ty: Type {
                ident: "test",
                generics: &[Generic::Type { ident: "A" }, Generic::Type { ident: "B" }],
            },
            fields: StructFields::Transparent {
                value: QualifiedType {
                    ns: "Ns",
                    ident: "inner",
                    generics: &[Generic::Type { ident: "A" }, Generic::Type { ident: "B" }],
                },
            },
        };

        assert_eq!(
            def.to_zod_string(),
            "const test = (A: z.ZodTypeAny, B: z.ZodTypeAny) => z.lazy(() => Ns.inner(A, B))"
        );

        assert_eq!(def.to_ts_string(), "type test<A, B> = Ns.inner<A, B>")
    }
}
