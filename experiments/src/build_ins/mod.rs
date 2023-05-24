use crate::{
    types::{ZodBool, ZodExport, ZodNumber, ZodString, ZodType},
    IoType,
};

const NAMESPACE: &'static str = "Rs";

/// Capitalizes the first character in s.
pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

macro_rules! impl_number {
    ($ident: ident, $suffix: expr) => {
        impl IoType for $ident {
            fn get_ref() -> $crate::types::ZodType {
                $crate::Reference::builder()
                    .ns(NAMESPACE)
                    .name(capitalize(stringify!($ident)))
                    .build()
                    .into()
            }

            fn visit_exports(set: &mut std::collections::HashSet<crate::types::ZodExport>) {
                set.insert(
                    ZodExport::builder()
                        .ns(NAMESPACE)
                        .name(capitalize(stringify!($ident)))
                        .value(
                            ZodType::builder()
                                .inner(ZodNumber)
                                .custom_suffix($suffix)
                                .build(),
                        )
                        .build(),
                );
            }
        }
    };
}

impl_number!(
    u8,
    format!(".integer().nonnegative().max({max})", max = u8::MAX)
);

impl_number!(
    u16,
    format!(".integer().nonnegative().max({max})", max = u16::MAX)
);

impl_number!(
    u32,
    format!(".integer().nonnegative().max({max})", max = u32::MAX)
);

impl_number!(
    i8,
    format!(
        ".integer().min({min}).max({max})",
        max = i8::MAX,
        min = i8::MIN
    )
);

impl_number!(
    i16,
    format!(
        ".integer().min({min}).max({max})",
        max = i16::MAX,
        min = i8::MIN
    )
);

impl_number!(
    i32,
    format!(
        ".integer().min({min}).max({max})",
        max = i32::MAX,
        min = i8::MIN
    )
);

impl IoType for bool {
    fn get_ref() -> crate::types::ZodType {
        crate::Reference::builder()
            .ns("Rs")
            .name("Bool")
            .build()
            .into()
    }

    fn visit_exports(set: &mut std::collections::HashSet<crate::types::ZodExport>) {
        set.insert(
            ZodExport::builder()
                .ns(NAMESPACE)
                .name("Bool")
                .value(ZodBool)
                .build(),
        );
    }
}

impl IoType for String {
    fn get_ref() -> crate::types::ZodType {
        crate::Reference::builder()
            .ns("Rs")
            .name("String")
            .build()
            .into()
    }
    fn visit_exports(set: &mut std::collections::HashSet<crate::types::ZodExport>) {
        set.insert(
            ZodExport::builder()
                .ns(NAMESPACE)
                .name("String")
                .value(ZodString)
                .build(),
        );
    }
}
