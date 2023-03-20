use std::collections::HashMap;
use std::fmt::Display;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ZodNode {
    ns: &'static str,
    name: &'static str,
    export: ZodExport,
    inline: &'static str,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ZodExport {
    zod: String,
}

impl Display for ZodNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.export.zod)?;
        Ok(())
    }
}

impl ZodNode {
    pub fn ns(&self) -> &'static str {
        self.ns
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

pub enum Generics {
    Literal(String),
    Map(HashMap<usize, Generics>),
}

pub trait ZodItem {
    const NS: &'static str;
    const NAME: &'static str;

    fn export() -> ZodExport;

    fn compose_zod(_: &Generics) -> String;
}

#[cfg(test)]
mod test {
    use super::*;

    impl ZodItem for String {
        const NS: &'static str = "Rs";
        const NAME: &'static str = "String";

        fn export() -> ZodExport {
            ZodExport {
                zod: String::from("export const String = z.string();"),
            }
        }

        fn compose_zod(generics: &Generics) -> String {
            if let Generics::Literal(ty) = generics {
                ty.clone()
            } else {
                String::from("Rs.String")
            }
        }
    }

    impl ZodItem for usize {
        const NS: &'static str = "Rs";
        const NAME: &'static str = "Usize";

        fn export() -> ZodExport {
            ZodExport {
                zod: String::from("export const Usize = z.number();"),
            }
        }

        fn compose_zod(generics: &Generics) -> String {
            if let Generics::Literal(ty) = generics {
                ty.clone()
            } else {
                String::from("Rs.Usize")
            }
        }
    }

    struct Generic<T1, T2> {
        inner1: T1,
        inner2: T2,
    }

    impl<T1: ZodItem, T2: ZodItem> ZodItem for Generic<T1, T2> {
        const NS: &'static str = "Custom";
        const NAME: &'static str = "Generic";

        fn export() -> ZodExport {
            ZodExport {
                zod: String::from(
                    "export const Generic = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.object({ inner1: T1, inner2: T2 })",
                ),
            }
        }

        fn compose_zod(generics: &Generics) -> String {
            match generics {
                Generics::Literal(lit) => lit.clone(),
                Generics::Map(map) => {
                    let t1 = if let Some(inner) = map.get(&0) {
                        T1::compose_zod(inner)
                    } else {
                        let inner = Generics::Map(Default::default());
                        T1::compose_zod(&inner)
                    };

                    let t2 = if let Some(inner) = map.get(&1) {
                        T2::compose_zod(inner)
                    } else {
                        let inner = Generics::Map(Default::default());
                        T2::compose_zod(&inner)
                    };

                    format!("{}.{}({}, {})", Self::NS, Self::NAME, t1, t2)
                }
            }
        }
    }

    struct MyType<T> {
        inner: Generic<String, T>,
    }

    impl<T: ZodItem> ZodItem for MyType<T> {
        const NS: &'static str = "Custom";
        const NAME: &'static str = "MyType";

        fn export() -> ZodExport {
            let mut map = HashMap::new();
            map.insert(1, Generics::Literal(String::from("T")));
            let generics = Generics::Map(map);

            ZodExport {
                zod: format!(
                    "export const MyType = (T: z.ZodTypeAny) => z.object({{ inner: {} }})",
                    Generic::<String, T>::compose_zod(&generics)
                ),
            }
        }

        fn compose_zod(generics: &Generics) -> String {
            match generics {
                Generics::Literal(inner) => inner.clone(),
                Generics::Map(map) => match map.get(&0) {
                    None => {
                        let default_map = Generics::Map(Default::default());
                        format!("Rs.MyType({})", T::compose_zod(&default_map))
                    }
                    Some(inner) => {
                        format!("Rs.MyType({})", T::compose_zod(inner))
                    }
                },
            }
        }
    }

    struct MyType2<T> {
        inner: Generic<String, Generic<T, usize>>,
    }
    impl<T: ZodItem> ZodItem for MyType2<T> {
        const NS: &'static str = "Custom";
        const NAME: &'static str = "MyType2";

        fn export() -> ZodExport {
            let mut map = HashMap::new();
            map.insert(
                1,
                Generics::Map({
                    let mut map = HashMap::new();
                    map.insert(0, Generics::Literal(String::from("T")));
                    map
                }),
            );
            let generics = Generics::Map(map);

            ZodExport {
                zod: format!(
                    "export const MyType2 = (T: z.ZodTypeAny) => z.object({{ inner: {} }})",
                    <Generic<String, Generic<String, T>>>::compose_zod(&generics)
                ),
            }
        }

        fn compose_zod(_: &Generics) -> String {
            format!(
                "Rs.MyType({})",
                T::compose_zod(&Generics::Map(Default::default()))
            )
        }
    }

    type Flat<T> = Generic<String, Generic<T, usize>>;

    struct MyType3<T> {
        inner: Flat<T>,
    }
    impl<T: ZodItem> ZodItem for MyType3<T> {
        const NS: &'static str = "Custom";
        const NAME: &'static str = "MyType2";

        fn export() -> ZodExport {
            let mut map = HashMap::new();
            map.insert(
                1,
                Generics::Map({
                    let mut map = HashMap::new();
                    map.insert(0, Generics::Literal(String::from("T")));
                    map
                }),
            );
            let generics = Generics::Map(map);

            ZodExport {
                zod: format!(
                    "export const MyType2 = (T: z.ZodTypeAny) => z.object({{ inner: {} }})",
                    <Flat<T>>::compose_zod(&generics)
                ),
            }
        }

        fn compose_zod(_: &Generics) -> String {
            format!(
                "Rs.MyType({})",
                T::compose_zod(&Generics::Map(Default::default()))
            )
        }
    }

    use pretty_assertions::assert_eq;

    #[test]
    fn ok() {
        assert_eq!(
            <MyType<usize>>::export().zod,
            "export const MyType = (T: z.ZodTypeAny) => z.object({ inner: Custom.Generic(Rs.String, T) })"
        );

        assert_eq!(
        <Generic<String, String>>::export().zod,
        "export const Generic = (T1: z.ZodTypeAny, T2: z.ZodTypeAny) => z.object({ inner1: T1, inner2: T2 })"
        );

        assert_eq!(
        <MyType2<usize>>::export().zod,
        "export const MyType2 = (T: z.ZodTypeAny) => z.object({ inner: Custom.Generic(Rs.String, Custom.Generic(T, Rs.Usize)) })"
        )
    }
}
