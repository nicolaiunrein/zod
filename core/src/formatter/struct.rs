use std::fmt::Display;

use super::{
    AnyNamedField, AnyTupleField, Delimited, FormatTypescript, FormatZod, StructFields, Type,
};

pub struct Struct {
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
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::formatter::{FlatField, FormatTypescript, Generic, NamedField, TupleField};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn zod_tuple_struct() {
        let def = Struct {
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
            ty: Type {
                ident: "test",
                generics: &[
                    Generic::Regular { ident: "A" },
                    Generic::Regular { ident: "B" },
                ],
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
                value: Type {
                    ident: "a",
                    generics: &[Generic::Regular { ident: "A" }],
                },
            }),
            AnyTupleField::Inner(TupleField {
                optional: false,
                value: Type {
                    ident: "b",
                    generics: &[Generic::Regular { ident: "B" }],
                },
            }),
            AnyTupleField::Inner(TupleField {
                optional: false,
                value: Type {
                    ident: "c",
                    generics: &[],
                },
            }),
            AnyTupleField::Inner(TupleField {
                optional: true,
                value: Type {
                    ident: "d",
                    generics: &[],
                },
            }),
        ];

        let def = Struct {
            ty: Type {
                ident: "test",
                generics: &[
                    Generic::Regular { ident: "A" },
                    Generic::Regular { ident: "B" },
                ],
            },
            fields: StructFields::Tuple(fields),
        };

        assert_eq!(
            def.to_zod_string(),
            "const test = (A: z.ZodTypeAny, B: z.ZodTypeAny) => z.lazy(() => z.tuple([a(A), b(B), c, d.optional()]));",
        );

        assert_eq!(
            def.to_ts_string(),
            format!(
                "type {} = [a<A>, b<B>, c, d | undefined];",
                def.ty.to_ts_string(),
            )
        );
    }

    #[test]
    fn zod_named_struct() {
        let def = Struct {
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
                value: Type {
                    ident: "a",
                    generics: &[Generic::Regular { ident: "A" }],
                },
            }),
            AnyNamedField::Inner(NamedField {
                optional: false,
                name: "hallo_b",
                value: Type {
                    ident: "b",
                    generics: &[Generic::Regular { ident: "B" }],
                },
            }),
            AnyNamedField::Inner(NamedField {
                optional: false,
                name: "hallo_c",
                value: Type {
                    ident: "c",
                    generics: &[],
                },
            }),
            AnyNamedField::Inner(NamedField {
                optional: true,
                name: "hallo_d",
                value: Type {
                    ident: "d",
                    generics: &[],
                },
            }),
            AnyNamedField::Flat(FlatField {
                value: Type {
                    ident: "e",
                    generics: &[],
                },
            }),
        ];

        let def = Struct {
            ty: Type {
                ident: "test",
                generics: &[
                    Generic::Regular { ident: "A" },
                    Generic::Regular { ident: "B" },
                ],
            },
            fields: StructFields::Named(fields),
        };

        assert_eq!(
            def.to_zod_string(),
            "const test = (A: z.ZodTypeAny, B: z.ZodTypeAny) => z.lazy(() => z.object({hallo_a: a(A), hallo_b: b(B), hallo_c: c, hallo_d: d.optional()})).extend(e);",
        );

        assert_eq!(
            def.to_ts_string(),
            "interface test<A, B> {hallo_a: a<A>, hallo_b: b<B>, hallo_c: c, hallo_d?: d | undefined} & e"
        );
    }
}
