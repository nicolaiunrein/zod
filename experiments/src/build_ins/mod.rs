use crate::{
    types::{ZodBool, ZodExport, ZodNumber, ZodString, ZodType},
    ExportVisitor, ReprDe, ReprSer,
};

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
        impl ReprSer for $ident {
            fn repr_ser() -> crate::Reference {
                crate::Reference::builder()
                    .name(capitalize(stringify!($ident)))
                    .build()
            }
        }

        impl ReprDe for $ident {
            fn repr_de() -> crate::Reference {
                crate::Reference::builder()
                    .name(capitalize(stringify!($ident)))
                    .build()
            }
        }

        impl ExportVisitor for $ident {
            fn visit_exports(set: &mut std::collections::HashSet<crate::types::ZodExport>) {
                set.insert(
                    ZodExport::builder()
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

impl ReprSer for bool {
    fn repr_ser() -> crate::Reference {
        crate::Reference::builder().name("Bool").build()
    }
}

impl ReprDe for bool {
    fn repr_de() -> crate::Reference {
        crate::Reference::builder().name("Bool").build()
    }
}

impl ExportVisitor for bool {
    fn visit_exports(set: &mut std::collections::HashSet<crate::types::ZodExport>) {
        set.insert(ZodExport::builder().name("Bool").value(ZodBool).build());
    }
}

impl ReprSer for String {
    fn repr_ser() -> crate::Reference {
        crate::Reference::builder().name("String").build()
    }
}

impl ReprDe for String {
    fn repr_de() -> crate::Reference {
        crate::Reference::builder().name("String").build()
    }
}

impl ExportVisitor for String {
    fn visit_exports(set: &mut std::collections::HashSet<crate::types::ZodExport>) {
        set.insert(ZodExport::builder().name("String").value(ZodString).build());
    }
}
