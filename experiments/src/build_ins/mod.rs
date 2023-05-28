use crate::{
    types::{Role, ZodBool, ZodExport, ZodNumber, ZodString, ZodType},
    InputType, Namespace, OutputType,
};

const NAMESPACE: &'static str = "Rs";

pub struct Rs;

impl Namespace for Rs {
    const NAME: &'static str = "Rs";
}

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
        impl InputType for $ident {
            type Namespace = Rs;
            fn get_input_ref() -> $crate::types::ZodType {
                $crate::Reference::builder()
                    .ns(Rs::NAME)
                    .name(capitalize(stringify!($ident)))
                    .role(Role::InputOnly)
                    .build()
                    .into()
            }

            fn visit_input_exports(set: &mut std::collections::HashSet<crate::types::ZodExport>) {
                set.insert(
                    ZodExport::builder()
                        .ns(NAMESPACE)
                        .name(capitalize(stringify!($ident)))
                        .context(Role::InputOnly)
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
        impl OutputType for $ident {
            type Namespace = Rs;
            fn get_output_ref() -> $crate::types::ZodType {
                $crate::Reference::builder()
                    .ns(Rs::NAME)
                    .name(capitalize(stringify!($ident)))
                    .role(Role::InputOnly)
                    .build()
                    .into()
            }

            fn visit_output_exports(set: &mut std::collections::HashSet<crate::types::ZodExport>) {
                set.insert(
                    ZodExport::builder()
                        .ns(NAMESPACE)
                        .name(capitalize(stringify!($ident)))
                        .context(Role::InputOnly)
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

impl InputType for bool {
    type Namespace = Rs;
    fn get_input_ref() -> crate::types::ZodType {
        crate::Reference::builder()
            .ns("Rs")
            .name("Bool")
            .role(Role::InputOnly)
            .build()
            .into()
    }

    fn visit_input_exports(set: &mut std::collections::HashSet<crate::types::ZodExport>) {
        set.insert(
            ZodExport::builder()
                .ns(NAMESPACE)
                .name("Bool")
                .context(Role::InputOnly)
                .value(ZodBool)
                .build(),
        );
    }
}

impl OutputType for bool {
    type Namespace = Rs;
    fn get_output_ref() -> crate::types::ZodType {
        crate::Reference::builder()
            .ns("Rs")
            .name("Bool")
            .role(Role::InputOnly)
            .build()
            .into()
    }

    fn visit_output_exports(set: &mut std::collections::HashSet<crate::types::ZodExport>) {
        set.insert(
            ZodExport::builder()
                .ns(NAMESPACE)
                .name("Bool")
                .context(Role::InputOnly)
                .value(ZodBool)
                .build(),
        );
    }
}

impl InputType for String {
    type Namespace = Rs;
    fn get_input_ref() -> crate::types::ZodType {
        crate::Reference::builder()
            .ns("Rs")
            .name("String")
            .role(Role::InputOnly)
            .build()
            .into()
    }
    fn visit_input_exports(set: &mut std::collections::HashSet<crate::types::ZodExport>) {
        set.insert(
            ZodExport::builder()
                .ns(NAMESPACE)
                .name("String")
                .context(Role::InputOnly)
                .value(ZodString)
                .build(),
        );
    }
}

impl OutputType for String {
    type Namespace = Rs;
    fn get_output_ref() -> crate::types::ZodType {
        crate::Reference::builder()
            .ns("Rs")
            .name("String")
            .role(Role::InputOnly)
            .build()
            .into()
    }
    fn visit_output_exports(set: &mut std::collections::HashSet<crate::types::ZodExport>) {
        set.insert(
            ZodExport::builder()
                .ns(NAMESPACE)
                .name("String")
                .context(Role::InputOnly)
                .value(ZodString)
                .build(),
        );
    }
}
