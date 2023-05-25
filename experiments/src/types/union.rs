use std::fmt::Display;

use typed_builder::TypedBuilder;

use crate::utils::Separated;

use super::{Ts, Zod, ZodType, ZodTypeInner};

#[derive(TypedBuilder, PartialEq, Eq, Debug, Clone, Hash)]
pub struct ZodUnion {
    #[builder(default)]
    pub variants: Vec<ZodType>,
}

impl Display for Zod<'_, ZodUnion> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(Zod).collect::<Vec<_>>();
        f.write_fmt(format_args!("z.union([{}])", Separated(", ", &variants)))
    }
}

impl Display for Ts<'_, ZodUnion> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variants = self.variants.iter().map(Ts).collect::<Vec<_>>();
        f.write_fmt(format_args!("{}", Separated(" | ", &variants)))
    }
}

impl From<ZodUnion> for ZodTypeInner {
    fn from(value: ZodUnion) -> Self {
        ZodTypeInner::Union(value)
    }
}

#[cfg(test)]
mod test {
    use crate::types::{ZodNumber, ZodString};

    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn fmt_ok() {
        assert_eq!(
            Zod(&ZodUnion::builder()
                .variants(vec![ZodString.into(), ZodNumber.into()])
                .build())
            .to_string(),
            "z.union([z.string(), z.number()])"
        );

        assert_eq!(
            Ts(&ZodUnion::builder()
                .variants(vec![ZodString.into(), ZodNumber.into()])
                .build())
            .to_string(),
            "string | number"
        );
    }
}
