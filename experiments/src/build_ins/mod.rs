use crate::{
    kind::Input,
    kind::Output,
    types::{ZodBool, ZodNumber, ZodString, ZodType},
    Namespace, Type,
};
use paste::paste;

pub struct Rs;

impl Namespace for Rs {
    const NAME: &'static str = "Rs";
}

macro_rules! impl_number {
    ($ident: ident, $suffix: expr) => {
        impl Type<Input> for $ident {
            type Ns = Rs;
            const NAME: &'static str = paste!(stringify!([<$ident:upper>]));
            fn value() -> ZodType<Input> {
                ZodType::builder()
                    .inner(ZodNumber)
                    .custom_suffix($suffix)
                    .build()
            }
        }
        impl Type<Output> for $ident {
            type Ns = Rs;
            const NAME: &'static str = paste!(stringify!([<$ident:upper>]));
            fn value() -> ZodType<Output> {
                ZodType::builder()
                    .inner(ZodNumber)
                    .custom_suffix($suffix)
                    .build()
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

impl Type<Input> for bool {
    type Ns = Rs;
    const NAME: &'static str = "Bool";
    fn value() -> ZodType<Input> {
        ZodBool.into()
    }
}

impl Type<Output> for bool {
    type Ns = Rs;
    const NAME: &'static str = "Bool";
    fn value() -> ZodType<Output> {
        ZodBool.into()
    }
}

impl Type<Input> for String {
    type Ns = Rs;
    const NAME: &'static str = "String";
    fn value() -> ZodType<Input> {
        ZodString.into()
    }
}

impl Type<Output> for String {
    type Ns = Rs;
    const NAME: &'static str = "String";
    fn value() -> ZodType<Output> {
        ZodString.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn names_ok() {
        assert_eq!(<u8 as Type<Input>>::NAME, "U8");
    }
}
