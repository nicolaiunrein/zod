use super::{
    AnyNamedField, AnyTupleField, FormatTypescript, FormatZod, GenericFunctionParams, StructFields,
    Type,
};

pub struct Struct {
    pub ty: Type,
    pub fields: StructFields,
}

impl FormatZod for Struct {
    fn fmt_zod(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("const ")?;
        self.ty.fmt_zod(f)?;
        f.write_str(" = ")?;
        if !self.ty.generics.is_empty() {
            let fn_params = GenericFunctionParams::from(self.ty.generics);
            fn_params.fmt_zod(f)?;
            f.write_str(" => ")?;
        }

        match self.fields {
            StructFields::Named(fields) => {
                f.write_str("z.lazy(() => z.object({")?;
                for field in fields {
                    if let AnyNamedField::Inner(inner) = field {
                        inner.fmt_zod(f)?;
                    }
                }
                f.write_str("}));")?;
            }
            StructFields::Tuple(fields) => {
                f.write_str("z.lazy(() => z.tuple([")?;
                let mut iter = fields.iter().peekable();
                while let Some(field) = iter.next() {
                    if let AnyTupleField::Inner(inner) = field {
                        inner.fmt_zod(f)?;
                        if iter.peek().is_some() {
                            f.write_str(", ")?;
                        }
                    }
                }
                f.write_str("]));")?;
            }
            StructFields::Unit => todo!(),
        }
        Ok(())
    }
}

impl FormatTypescript for Struct {
    fn fmt_ts(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.fields {
            StructFields::Named(fields) => {
                let (inner, flat) = AnyNamedField::partition(fields);
                if inner.is_empty() {
                    f.write_str("interface ")?;
                    self.ty.fmt_ts(f)?;
                    f.write_str(" {")?;
                    // for ext in flat {
                    // f.write_str("& ")?;
                    // ext.fmt_ts(f)?;
                    // }
                    f.write_str("};")?;
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
                    for field in inner {
                        field.fmt_ts(f)?;
                    }
                    f.write_str("]")?;
                    for ext in flat {
                        f.write_str("& ")?;
                        ext.fmt_ts(f)?;
                    }
                    f.write_str(";")?;
                }
            }
            StructFields::Unit => todo!(),
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::formatter::{FormatTypescript, Generic, GenericTypeParams, TupleField};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn zod_tuple_struct() {
        let def = Struct {
            ty: Type {
                ident: "test",
                generics: GenericTypeParams::default(),
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
                generics: GenericTypeParams(&[
                    Generic::Regular { ident: "A" },
                    Generic::Regular { ident: "B" },
                ]),
            },
            fields: StructFields::Tuple(&[]),
        };

        assert_eq!(
            def.to_zod_string(),
            format!(
                "const {} = {} => z.lazy(() => z.tuple([]));",
                def.ty.to_zod_string(),
                GenericFunctionParams::from(def.ty.generics).to_zod_string(),
            )
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
                value: Type {
                    ident: "a",
                    generics: GenericTypeParams(&[Generic::Regular { ident: "A" }]),
                },
            }),
            AnyTupleField::Inner(TupleField {
                value: Type {
                    ident: "b",
                    generics: GenericTypeParams(&[Generic::Regular { ident: "B" }]),
                },
            }),
            AnyTupleField::Inner(TupleField {
                value: Type {
                    ident: "c",
                    generics: GenericTypeParams(&[]),
                },
            }),
        ];

        let def = Struct {
            ty: Type {
                ident: "test",
                generics: GenericTypeParams(&[
                    Generic::Regular { ident: "A" },
                    Generic::Regular { ident: "B" },
                ]),
            },
            fields: StructFields::Tuple(fields),
        };

        assert_eq!(
            def.to_zod_string(),
            format!(
                "const {} = {} => z.lazy(() => z.tuple([a(A), b(B), c]));",
                def.ty.to_zod_string(),
                GenericFunctionParams::from(def.ty.generics).to_zod_string(),
            )
        );

        assert_eq!(
            def.to_ts_string(),
            format!("type {} = [];", def.ty.to_ts_string(),)
        );
    }

    #[test]
    fn zod_named_struct() {
        let def = Struct {
            ty: Type {
                ident: "test",
                generics: GenericTypeParams::default(),
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

        // assert_eq!(
        // def.to_ts_string(),
        // format!("type {} = {{}};", def.ty.to_ts_string())
        // );
    }
}
