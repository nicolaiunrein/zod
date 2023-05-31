use crate::{
    types::{ZodBool, ZodNumber, ZodString, ZodType},
    DependencyVisitor, GenericArgument, Kind, Namespace, Type,
};
use paste::paste;

pub struct Rs;

impl Namespace for Rs {
    const NAME: &'static str = "Rs";
}

macro_rules! impl_number {
    ($ident: ident, $suffix: expr) => {
        impl Type<Kind::Input> for $ident {
            type Ns = Rs;
            const NAME: &'static str = paste!(stringify!([<$ident:upper>]));
            const INLINE: bool = false;
            fn value() -> ZodType<Kind::Input> {
                ZodType::builder()
                    .inner(ZodNumber)
                    .custom_suffix($suffix)
                    .build()
            }
            fn visit_dependencies(_visitor: &mut DependencyVisitor<Kind::Input>) {}
            fn args() -> Vec<GenericArgument<Kind::Input>> {
                Vec::new()
            }
        }
        impl Type<Kind::Output> for $ident {
            type Ns = Rs;
            const NAME: &'static str = paste!(stringify!([<$ident:upper>]));
            const INLINE: bool = false;
            fn value() -> ZodType<Kind::Output> {
                ZodType::builder()
                    .inner(ZodNumber)
                    .custom_suffix($suffix)
                    .build()
            }
            fn visit_dependencies(_visitor: &mut DependencyVisitor<Kind::Output>) {}
            fn args() -> Vec<GenericArgument<Kind::Output>> {
                Vec::new()
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

impl Type<Kind::Input> for bool {
    type Ns = Rs;
    const NAME: &'static str = "Bool";
    const INLINE: bool = false;
    fn value() -> ZodType<Kind::Input> {
        ZodBool.into()
    }
    fn visit_dependencies(_visitor: &mut DependencyVisitor<Kind::Input>) {}

    fn args() -> Vec<GenericArgument<Kind::Input>> {
        Vec::new()
    }
}

impl Type<Kind::Output> for bool {
    type Ns = Rs;
    const NAME: &'static str = "Bool";
    const INLINE: bool = false;
    fn value() -> ZodType<Kind::Output> {
        ZodBool.into()
    }
    fn visit_dependencies(_visitor: &mut DependencyVisitor<Kind::Output>) {}
    fn args() -> Vec<GenericArgument<Kind::Output>> {
        Vec::new()
    }
}

impl Type<Kind::Input> for String {
    type Ns = Rs;
    const NAME: &'static str = "String";
    const INLINE: bool = false;
    fn value() -> ZodType<Kind::Input> {
        ZodString.into()
    }
    fn visit_dependencies(_visitor: &mut DependencyVisitor<Kind::Input>) {}
    fn args() -> Vec<GenericArgument<Kind::Input>> {
        Vec::new()
    }
}

impl Type<Kind::Output> for String {
    type Ns = Rs;
    const NAME: &'static str = "String";
    const INLINE: bool = false;
    fn value() -> ZodType<Kind::Output> {
        ZodString.into()
    }
    fn visit_dependencies(_visitor: &mut DependencyVisitor<Kind::Output>) {}
    fn args() -> Vec<GenericArgument<Kind::Output>> {
        Vec::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn names_ok() {
        assert_eq!(<u8 as Type<Kind::Input>>::NAME, "U8");
    }
}
